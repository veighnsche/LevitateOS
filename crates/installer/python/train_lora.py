#!/usr/bin/env python3
"""
Train a LoRA adapter for the LevitateOS installer LLM.

CRITICAL: This trainer expects MULTI-TURN CONVERSATION data.
Each training example has a 'messages' array with full conversation history.

Input format (JSONL):
{
  "system_context": "## Current System State...",
  "messages": [
    {"role": "user", "content": "list disks"},
    {"role": "assistant", "content": "$ lsblk\n\nNAME  SIZE\nsda   500G"},
    {"role": "user", "content": "partition it"}
  ],
  "expected_response": {"type": "command", "command": "sgdisk -Z /dev/sda"}
}

Usage:
    python train_lora.py --model vendor/models/SmolLM3-3B --output adapters/installer
"""

import argparse
import json
import sys
from pathlib import Path

import torch

# Script directory for resolving relative paths
SCRIPT_DIR = Path(__file__).parent.resolve()
from datasets import Dataset
from peft import LoraConfig, get_peft_model, TaskType, prepare_model_for_kbit_training
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    TrainingArguments,
    Trainer,
    DataCollatorForLanguageModeling,
    BitsAndBytesConfig,
)


# Tool definition for SmolLM3-3B
SHELL_COMMAND_TOOL = {
    "type": "function",
    "function": {
        "name": "run_shell_command",
        "description": "Execute a shell command for system installation tasks.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "The shell command to execute"}
            },
            "required": ["command"]
        }
    }
}

# System prompt template - optimized for tool call consistency
SYSTEM_PROMPT_TEMPLATE = """You are the LevitateOS installation assistant. Help users install their operating system.

{system_context}

CRITICAL RULES:
1. When user wants to DO something (list, format, partition, mount, create, set, install), ALWAYS call run_shell_command
2. When user CONFIRMS an action (yes, ok, proceed, continue, do it), EXECUTE the pending command via run_shell_command
3. When user asks a QUESTION (what is, how do, should I, explain), respond with text

COMMAND REFERENCE:
- List disks: lsblk
- Partition disk: sgdisk -Z /dev/X && sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 /dev/X
- Format EFI: mkfs.fat -F32 /dev/X1
- Format root: mkfs.ext4 /dev/X2
- Mount root: mount /dev/X2 /mnt
- Mount EFI: mkdir -p /mnt/boot/efi && mount /dev/X1 /mnt/boot/efi
- Set hostname: hostnamectl set-hostname NAME
- Set timezone: timedatectl set-timezone ZONE
- Create user: useradd -m -G wheel NAME
- Install GRUB: grub-install --target=x86_64-efi --efi-directory=/boot/efi

Only reference disks that exist in the system state above. Never hallucinate disk names."""

# Default context for examples without explicit context
DEFAULT_SYSTEM_CONTEXT = """## Current System State

- Boot mode: UEFI
- Network: Connected
- Hostname: archiso
- Timezone: not set

## Available Disks

- /dev/sda: 500G (Samsung SSD 870)"""


def load_training_data(data_dir: Path) -> list[dict]:
    """Load all JSONL training files from directory."""
    all_examples = []

    for jsonl_file in data_dir.glob("*.jsonl"):
        print(f"Loading {jsonl_file.name}...")
        with open(jsonl_file) as f:
            for line_num, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    example = json.loads(line)
                    # Validate format
                    if "messages" not in example:
                        print(f"  Warning: {jsonl_file.name}:{line_num} - missing 'messages' field, skipping")
                        continue
                    if "expected_response" not in example:
                        print(f"  Warning: {jsonl_file.name}:{line_num} - missing 'expected_response' field, skipping")
                        continue
                    all_examples.append(example)
                except json.JSONDecodeError as e:
                    print(f"  Skipping invalid JSON at line {line_num}: {e}", file=sys.stderr)

    print(f"Loaded {len(all_examples)} training examples")
    return all_examples


def format_example_for_training(example: dict, tokenizer) -> dict:
    """
    Format a CONVERSATION training example into SmolLM3 chat format.

    Returns both the full text AND the prompt (everything except the final assistant response)
    so we can properly mask the prompt during training.

    The example contains:
    - system_context: System state info
    - messages: Array of user/assistant messages (the conversation history)
    - expected_response: What the LLM should output for the final user message

    SmolLM3 format:
    - Uses "system" role with /no_think prefix for fast mode
    - Tool calls returned as: <tool_call>{"name": ..., "arguments": ...}</tool_call>
    - Uses <|im_start|> and <|im_end|> tokens
    """
    # Build system prompt with context
    system_context = example.get("system_context", DEFAULT_SYSTEM_CONTEXT)
    system_prompt = "/no_think " + SYSTEM_PROMPT_TEMPLATE.format(system_context=system_context)

    # Build messages array: system + conversation history
    messages = [{"role": "system", "content": system_prompt}]

    # Add all conversation messages from history
    for msg in example["messages"]:
        if msg["role"] == "assistant" and msg["content"].startswith("$ "):
            # This is a command result in history - keep as content
            messages.append({"role": "assistant", "content": msg["content"]})
        else:
            messages.append({"role": msg["role"], "content": msg["content"]})

    # Build the expected response message
    expected = example["expected_response"]
    if expected["type"] == "command":
        # SmolLM3 uses XML-style tool calls: <tool_call>{"name": ..., "arguments": ...}</tool_call>
        import json
        tool_call_json = json.dumps({
            "name": "run_shell_command",
            "arguments": {"command": expected["command"]}
        })
        assistant_message = {
            "role": "assistant",
            "content": f"<tool_call>\n{tool_call_json}\n</tool_call>"
        }
    else:
        # Text response - use regular content
        assistant_message = {
            "role": "assistant",
            "content": expected.get("response", "")
        }

    # Get prompt text (WITHOUT the final assistant response) for loss masking
    try:
        prompt_text = tokenizer.apply_chat_template(
            messages,
            tools=[SHELL_COMMAND_TOOL],
            tokenize=False,
            add_generation_prompt=True
        )
    except Exception as e:
        print(f"Warning: chat template failed for prompt: {e}", file=sys.stderr)
        # Fallback for SmolLM3 format
        parts = [f"<|im_start|>system\n{system_prompt}<|im_end|>\n"]
        for msg in example["messages"]:
            parts.append(f"<|im_start|>{msg['role']}\n{msg['content']}<|im_end|>\n")
        parts.append("<|im_start|>assistant\n")
        prompt_text = "".join(parts)

    # Get full text (WITH the assistant response) for training
    messages_with_response = messages + [assistant_message]
    try:
        full_text = tokenizer.apply_chat_template(
            messages_with_response,
            tools=[SHELL_COMMAND_TOOL],
            tokenize=False,
            add_generation_prompt=False
        )
    except Exception as e:
        print(f"Warning: chat template failed for full text: {e}", file=sys.stderr)
        # Fallback - manually construct
        import json
        if expected["type"] == "command":
            tool_call_json = json.dumps({
                "name": "run_shell_command",
                "arguments": {"command": expected["command"]}
            })
            response_text = f"<tool_call>\n{tool_call_json}\n</tool_call>"
        else:
            response_text = expected.get("response", "")
        full_text = prompt_text + response_text + "<|im_end|>\n"

    return {"full_text": full_text, "prompt_text": prompt_text}


def prepare_dataset(examples: list[dict], tokenizer, max_length: int = 512) -> Dataset:
    """Convert examples to HuggingFace Dataset with tokenization and loss masking.

    Loss masking ensures only the assistant response contributes to training loss.
    The prompt (system + user messages) is masked with -100.
    """

    # Format all examples
    formatted = [format_example_for_training(ex, tokenizer) for ex in examples]

    # Create dataset
    dataset = Dataset.from_list(formatted)

    # Tokenize with loss masking
    def tokenize_with_masking(example):
        # Tokenize the full text (what the model sees)
        full_tokens = tokenizer(
            example["full_text"],
            truncation=True,
            max_length=max_length,
            padding="max_length",
            return_tensors=None
        )

        # Tokenize just the prompt to find where response starts
        prompt_tokens = tokenizer(
            example["prompt_text"],
            truncation=True,
            max_length=max_length,
            add_special_tokens=False,  # Avoid double-counting special tokens
            return_tensors=None
        )

        # The prompt length tells us where to start computing loss
        prompt_len = len(prompt_tokens["input_ids"])

        # Create labels: -100 for prompt tokens (masked), actual ids for response
        labels = [-100] * len(full_tokens["input_ids"])
        for i in range(prompt_len, len(full_tokens["input_ids"])):
            # Only set label if not padding
            if full_tokens["input_ids"][i] != tokenizer.pad_token_id:
                labels[i] = full_tokens["input_ids"][i]

        return {
            "input_ids": full_tokens["input_ids"],
            "attention_mask": full_tokens["attention_mask"],
            "labels": labels
        }

    tokenized = dataset.map(
        tokenize_with_masking,
        remove_columns=["full_text", "prompt_text"],
        desc="Tokenizing with loss masking"
    )

    return tokenized


def main():
    # Default paths relative to script location
    default_data_dir = SCRIPT_DIR / "training"
    default_output = SCRIPT_DIR / "adapters" / "installer"

    parser = argparse.ArgumentParser(description="Train LoRA adapter for installer LLM")
    parser.add_argument("--model", "-m", default="vendor/models/SmolLM3-3B",
                        help="Base model path")
    parser.add_argument("--output", "-o", default=str(default_output),
                        help="Output directory for LoRA adapter")
    parser.add_argument("--data-dir", "-d", default=str(default_data_dir),
                        help="Directory containing training JSONL files")
    parser.add_argument("--epochs", type=int, default=5, help="Number of training epochs")
    parser.add_argument("--batch-size", type=int, default=1, help="Training batch size (default 1 for memory)")
    parser.add_argument("--learning-rate", type=float, default=1e-4, help="Learning rate")
    parser.add_argument("--lora-r", type=int, default=32, help="LoRA rank (higher = more capacity)")
    parser.add_argument("--lora-alpha", type=int, default=64, help="LoRA alpha (typically 2x rank)")
    parser.add_argument("--max-length", type=int, default=768, help="Max sequence length")
    parser.add_argument("--use-4bit", action="store_true", help="Use 4-bit quantization (saves memory)")
    parser.add_argument("--use-8bit", action="store_true", help="Use 8-bit quantization (saves memory)")
    parser.add_argument("--cpu", action="store_true", help="Force CPU training (slow but works)")
    parser.add_argument("--no-gradient-checkpointing", action="store_true",
                        help="Disable gradient checkpointing (uses more memory)")
    args = parser.parse_args()

    # Resolve model path
    model_path = Path(args.model)
    if not model_path.is_absolute() and not model_path.exists():
        # Try relative to project root
        project_root = SCRIPT_DIR.parent.parent.parent
        model_path = project_root / args.model
    model_path = model_path.resolve()

    # Resolve data directory
    data_dir = Path(args.data_dir).resolve()

    # Resolve output directory
    output_dir = Path(args.output).resolve()

    if not model_path.exists():
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    if not data_dir.exists():
        print(f"Error: Training data directory not found at {data_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Model: {model_path}")
    print(f"Data: {data_dir}")
    print(f"Output: {output_dir}")

    # Load training data
    print("\n=== Loading training data ===")
    examples = load_training_data(data_dir)

    if not examples:
        print("Error: No training examples found", file=sys.stderr)
        sys.exit(1)

    # Validate data format
    sample = examples[0]
    print(f"\nSample training example:")
    print(f"  Messages in history: {len(sample['messages'])}")
    print(f"  Last user message: {sample['messages'][-1]['content'][:50]}...")
    print(f"  Expected response type: {sample['expected_response']['type']}")

    # Load tokenizer
    print("\n=== Loading tokenizer ===")
    tokenizer = AutoTokenizer.from_pretrained(model_path)

    # Ensure padding token
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token
        tokenizer.pad_token_id = tokenizer.eos_token_id

    # Prepare dataset
    print("\n=== Preparing dataset ===")
    dataset = prepare_dataset(examples, tokenizer, max_length=args.max_length)
    print(f"Dataset size: {len(dataset)} examples")

    # Split into train/eval (90/10)
    split = dataset.train_test_split(test_size=0.1, seed=42)
    train_dataset = split["train"]
    eval_dataset = split["test"]
    print(f"Train: {len(train_dataset)}, Eval: {len(eval_dataset)}")

    # Load model
    print("\n=== Loading model ===")

    if args.cpu:
        print("Using CPU (this will be slow)...")
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=torch.float32,
            device_map={"": "cpu"},
            trust_remote_code=True,
        )
    elif args.use_4bit:
        print("Using 4-bit quantization...")
        bnb_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.float16,
            bnb_4bit_use_double_quant=True,
        )
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            quantization_config=bnb_config,
            device_map={"": 0} if torch.cuda.is_available() else {"": "cpu"},
            trust_remote_code=True,
        )
        model = prepare_model_for_kbit_training(model)
    elif args.use_8bit:
        print("Using 8-bit quantization...")
        bnb_config = BitsAndBytesConfig(
            load_in_8bit=True,
        )
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            quantization_config=bnb_config,
            device_map={"": 0} if torch.cuda.is_available() else {"": "cpu"},
            trust_remote_code=True,
        )
        model = prepare_model_for_kbit_training(model)
    else:
        print("Using full precision (may OOM on GPU)...")
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=torch.float32,
            device_map={"": 0} if torch.cuda.is_available() else {"": "cpu"},
            trust_remote_code=True,
        )

    # Enable gradient checkpointing by default to save memory
    if not args.no_gradient_checkpointing:
        model.gradient_checkpointing_enable()
        print("Gradient checkpointing enabled.", file=sys.stderr)

    # Configure LoRA
    print("\n=== Configuring LoRA ===")
    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=args.lora_r,
        lora_alpha=args.lora_alpha,
        lora_dropout=0.05,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"],
        bias="none",
    )

    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    # Training arguments
    print("\n=== Setting up training ===")

    effective_batch_size = 8
    gradient_accumulation = effective_batch_size // args.batch_size

    training_args = TrainingArguments(
        output_dir=str(output_dir),
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        per_device_eval_batch_size=args.batch_size,
        learning_rate=args.learning_rate,
        weight_decay=0.01,
        logging_steps=50,
        eval_strategy="epoch",
        save_strategy="epoch",
        load_best_model_at_end=True,
        metric_for_best_model="eval_loss",
        greater_is_better=False,
        warmup_ratio=0.1,
        lr_scheduler_type="cosine",
        fp16=torch.cuda.is_available() and not args.use_4bit and not args.cpu,
        bf16=False,
        use_cpu=args.cpu,
        gradient_accumulation_steps=gradient_accumulation,
        gradient_checkpointing=not args.no_gradient_checkpointing,
        optim="adamw_torch",
        report_to="none",
        remove_unused_columns=False,
        dataloader_pin_memory=False,
        max_grad_norm=1.0,
    )

    print(f"Batch size: {args.batch_size}, Gradient accumulation: {gradient_accumulation}")

    # Data collator
    data_collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer,
        mlm=False,
    )

    # Trainer
    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=eval_dataset,
        data_collator=data_collator,
    )

    # Train
    print("\n=== Starting training ===")
    trainer.train()

    # Save the LoRA adapter
    print(f"\n=== Saving LoRA adapter to {output_dir} ===")
    model.save_pretrained(output_dir)
    tokenizer.save_pretrained(output_dir)

    print("\nTraining complete!")
    print(f"LoRA adapter saved to: {output_dir}")
    print(f"\nTo use the adapter, load the base model and apply the adapter:")
    print(f"  from peft import PeftModel")
    print(f"  model = AutoModelForCausalLM.from_pretrained('{model_path}')")
    print(f"  model = PeftModel.from_pretrained(model, '{output_dir}')")


if __name__ == "__main__":
    main()
