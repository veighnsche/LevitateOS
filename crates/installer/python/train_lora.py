#!/usr/bin/env python3
"""
Train a LoRA adapter for the LevitateOS installer LLM.

Usage:
    python train_lora.py --model vendor/models/FunctionGemma --output adapters/installer
"""

import argparse
import json
import os
import sys
from pathlib import Path

import torch
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


# Tool definition for FunctionGemma
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

# System prompt template - {system_context} gets replaced with actual disk/system info
SYSTEM_PROMPT_TEMPLATE = """You are the LevitateOS installation assistant. Help users install their operating system.

You can:
- List and partition disks
- Configure system settings (hostname, timezone, language, keyboard)
- Create user accounts
- Install the bootloader

{system_context}

IMPORTANT: Only reference disks and partitions that actually exist in the system state above.
Do NOT make up or hallucinate disk names, sizes, or other system information.

When the user asks to perform an action, call run_shell_command with the appropriate command.
When the user asks a question or needs clarification, respond in natural language using the facts above."""

# Fallback context for examples without explicit context
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
            for line in f:
                line = line.strip()
                if line:
                    try:
                        example = json.loads(line)
                        all_examples.append(example)
                    except json.JSONDecodeError as e:
                        print(f"  Skipping invalid JSON: {e}", file=sys.stderr)

    print(f"Loaded {len(all_examples)} training examples")
    return all_examples


def format_example_for_training(example: dict, tokenizer) -> dict:
    """
    Format a training example into the chat format expected by FunctionGemma.

    Each example becomes a conversation:
    - System prompt (with injected system context)
    - User query
    - Assistant response (either function call or text)
    """
    # Get system context from example, or use default
    system_context = example.get("system_context", DEFAULT_SYSTEM_CONTEXT)
    system_prompt = SYSTEM_PROMPT_TEMPLATE.format(system_context=system_context)

    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": example["query"]}
    ]

    # Determine the expected response
    if "command" in example:
        # This should produce a function call
        # FunctionGemma format: <start_function_call>call:run_shell_command{command:<escape>CMD<escape>}<end_function_call>
        assistant_content = f"<start_function_call>call:run_shell_command{{command:<escape>{example['command']}<escape>}}<end_function_call>"
    else:
        # Natural language response
        assistant_content = example.get("response", "")

    messages.append({"role": "assistant", "content": assistant_content})

    # Apply chat template
    try:
        text = tokenizer.apply_chat_template(
            messages,
            tools=[SHELL_COMMAND_TOOL],
            tokenize=False,
            add_generation_prompt=False
        )
    except Exception:
        # Fallback: simple concatenation if chat template fails
        text = f"<system>{system_prompt}</system>\n<user>{example['query']}</user>\n<assistant>{assistant_content}</assistant>"

    return {"text": text}


def prepare_dataset(examples: list[dict], tokenizer, max_length: int = 512) -> Dataset:
    """Convert examples to HuggingFace Dataset with tokenization."""

    # Format all examples
    formatted = [format_example_for_training(ex, tokenizer) for ex in examples]

    # Create dataset
    dataset = Dataset.from_list(formatted)

    # Tokenize
    def tokenize_fn(batch):
        return tokenizer(
            batch["text"],
            truncation=True,
            max_length=max_length,
            padding="max_length",
            return_tensors=None
        )

    tokenized = dataset.map(
        tokenize_fn,
        batched=True,
        remove_columns=["text"],
        desc="Tokenizing"
    )

    # Add labels (same as input_ids for causal LM)
    def add_labels(batch):
        batch["labels"] = batch["input_ids"].copy()
        return batch

    tokenized = tokenized.map(add_labels, batched=True)

    return tokenized


def main():
    parser = argparse.ArgumentParser(description="Train LoRA adapter for installer LLM")
    parser.add_argument("--model", "-m", default="vendor/models/FunctionGemma",
                        help="Base model path")
    parser.add_argument("--output", "-o", default="crates/installer/python/adapters/installer",
                        help="Output directory for LoRA adapter")
    parser.add_argument("--data-dir", "-d", default="crates/installer/python/training",
                        help="Directory containing training JSONL files")
    parser.add_argument("--epochs", type=int, default=3, help="Number of training epochs")
    parser.add_argument("--batch-size", type=int, default=1, help="Training batch size (default 1 for memory)")
    parser.add_argument("--learning-rate", type=float, default=2e-4, help="Learning rate")
    parser.add_argument("--lora-r", type=int, default=16, help="LoRA rank")
    parser.add_argument("--lora-alpha", type=int, default=32, help="LoRA alpha")
    parser.add_argument("--max-length", type=int, default=256, help="Max sequence length (shorter = less memory)")
    parser.add_argument("--use-4bit", action="store_true", help="Use 4-bit quantization (saves memory)")
    parser.add_argument("--use-8bit", action="store_true", help="Use 8-bit quantization (saves memory)")
    parser.add_argument("--cpu", action="store_true", help="Force CPU training (slow but works)")
    parser.add_argument("--no-gradient-checkpointing", action="store_true",
                        help="Disable gradient checkpointing (uses more memory)")
    args = parser.parse_args()

    model_path = Path(args.model)
    data_dir = Path(args.data_dir)
    output_dir = Path(args.output)

    if not model_path.exists():
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    if not data_dir.exists():
        print(f"Error: Training data directory not found at {data_dir}", file=sys.stderr)
        sys.exit(1)

    # Load training data
    print("\n=== Loading training data ===")
    examples = load_training_data(data_dir)

    if not examples:
        print("Error: No training examples found", file=sys.stderr)
        sys.exit(1)

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
        # 4-bit quantization config
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
        # 8-bit quantization config
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

    # Training arguments - optimized for memory
    print("\n=== Setting up training ===")

    # Use gradient accumulation to simulate larger batch sizes
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
        optim="adamw_torch",  # More memory efficient
        report_to="none",  # Disable wandb etc
        remove_unused_columns=False,
        dataloader_pin_memory=False,  # Save memory
        max_grad_norm=1.0,
    )

    print(f"Batch size: {args.batch_size}, Gradient accumulation: {gradient_accumulation}")

    # Data collator
    data_collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer,
        mlm=False,  # Causal LM, not masked LM
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
