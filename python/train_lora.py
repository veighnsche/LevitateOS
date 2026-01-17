#!/usr/bin/env python3
"""
LoRA Trainer - Fine-tune language models with LoRA adapters.

Supports multi-turn conversation training with proper loss masking
(only train on assistant responses, not prompts).

Input format (JSONL):
{
    "messages": [
        {"role": "system", "content": "You are..."},
        {"role": "user", "content": "Hello"},
        {"role": "assistant", "content": "Hi there!"}
    ]
}

Or with expected_response for training:
{
    "messages": [{"role": "user", "content": "Hello"}],
    "expected_response": {
        "type": "text",
        "response": "Hi there!",
        "thinking": "User is greeting me..."  // optional
    }
}

Usage:
    python train_lora.py --model path/to/model --data training.jsonl --output ./lora_adapter

    # With quantization (saves memory)
    python train_lora.py --model path/to/model --data training.jsonl --output ./lora_adapter --use-4bit

    # Custom hyperparameters
    python train_lora.py --model path/to/model --data training.jsonl --output ./lora_adapter \
        --epochs 5 --lora-r 32 --lora-alpha 64 --learning-rate 1e-4
"""

import argparse
import json
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


def load_training_data(data_path: Path) -> list[dict]:
    """Load training examples from JSONL file(s)."""
    all_examples = []

    paths = list(data_path.glob("*.jsonl")) if data_path.is_dir() else [data_path]

    for jsonl_file in paths:
        print(f"Loading {jsonl_file.name}...")
        with open(jsonl_file) as f:
            for line_num, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    example = json.loads(line)
                    if "messages" in example:
                        all_examples.append(example)
                    else:
                        print(f"  Warning: {jsonl_file.name}:{line_num} - missing 'messages', skipping")
                except json.JSONDecodeError as e:
                    print(f"  Skipping invalid JSON at line {line_num}: {e}", file=sys.stderr)

    print(f"Loaded {len(all_examples)} training examples")
    return all_examples


def format_example_for_training(example: dict, tokenizer, tools: list = None) -> dict:
    """
    Format a training example into chat format with loss masking.

    Returns both the full text and prompt text for loss masking.
    """
    messages = list(example.get("messages", []))

    # If we have expected_response, convert it to an assistant message
    if "expected_response" in example:
        expected = example["expected_response"]
        thinking = expected.get("thinking", "")

        if expected.get("type") == "command":
            # Format as tool call
            tool_call_json = json.dumps({
                "name": "run_shell_command",
                "arguments": {"command": expected["command"]}
            })
            if thinking:
                content = f"<think>\n{thinking}\n</think>\n\n<tool_call>\n{tool_call_json}\n</tool_call>"
            else:
                content = f"<tool_call>\n{tool_call_json}\n</tool_call>"
        else:
            response = expected.get("response", "")
            if thinking:
                content = f"<think>\n{thinking}\n</think>\n\n{response}"
            else:
                content = response

        assistant_message = {"role": "assistant", "content": content}
    else:
        # Find the last assistant message to use as the target
        assistant_message = None
        for i in range(len(messages) - 1, -1, -1):
            if messages[i]["role"] == "assistant":
                assistant_message = messages[i]
                messages = messages[:i]  # Prompt is everything before
                break

        if assistant_message is None:
            # No assistant message to train on
            return None

    # Build template kwargs
    template_kwargs = {
        "tokenize": False,
        "add_generation_prompt": True
    }
    if tools:
        template_kwargs["tools"] = tools

    # Get prompt text (without assistant response)
    try:
        prompt_text = tokenizer.apply_chat_template(messages, **template_kwargs)
    except Exception as e:
        print(f"Warning: chat template failed: {e}", file=sys.stderr)
        return None

    # Get full text (with assistant response)
    messages_with_response = messages + [assistant_message]
    template_kwargs["add_generation_prompt"] = False
    try:
        full_text = tokenizer.apply_chat_template(messages_with_response, **template_kwargs)
    except Exception as e:
        print(f"Warning: chat template failed for full text: {e}", file=sys.stderr)
        return None

    return {"full_text": full_text, "prompt_text": prompt_text}


def prepare_dataset(examples: list[dict], tokenizer, max_length: int = 512, tools: list = None) -> Dataset:
    """Convert examples to HuggingFace Dataset with tokenization and loss masking."""

    # Format all examples
    formatted = []
    for ex in examples:
        result = format_example_for_training(ex, tokenizer, tools)
        if result:
            formatted.append(result)

    print(f"Formatted {len(formatted)}/{len(examples)} examples")

    dataset = Dataset.from_list(formatted)

    # Tokenize with loss masking
    def tokenize_with_masking(example):
        full_tokens = tokenizer(
            example["full_text"],
            truncation=True,
            max_length=max_length,
            padding="max_length",
            return_tensors=None
        )

        prompt_tokens = tokenizer(
            example["prompt_text"],
            truncation=True,
            max_length=max_length,
            add_special_tokens=False,
            return_tensors=None
        )

        prompt_len = len(prompt_tokens["input_ids"])

        # Create labels: -100 for prompt tokens (masked), actual ids for response
        labels = [-100] * len(full_tokens["input_ids"])
        for i in range(prompt_len, len(full_tokens["input_ids"])):
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
    parser = argparse.ArgumentParser(description="Train LoRA adapter for language models")
    parser.add_argument("--model", "-m", required=True, help="Base model path or HuggingFace ID")
    parser.add_argument("--data", "-d", required=True, help="Training data (JSONL file or directory)")
    parser.add_argument("--output", "-o", required=True, help="Output directory for LoRA adapter")

    # Training hyperparameters
    parser.add_argument("--epochs", type=int, default=3, help="Number of training epochs")
    parser.add_argument("--batch-size", type=int, default=1, help="Training batch size")
    parser.add_argument("--learning-rate", type=float, default=1e-4, help="Learning rate")
    parser.add_argument("--max-length", type=int, default=512, help="Max sequence length")

    # LoRA hyperparameters
    parser.add_argument("--lora-r", type=int, default=16, help="LoRA rank")
    parser.add_argument("--lora-alpha", type=int, default=32, help="LoRA alpha (typically 2x rank)")
    parser.add_argument("--lora-dropout", type=float, default=0.05, help="LoRA dropout")
    parser.add_argument("--target-modules", nargs="+",
                        default=["q_proj", "k_proj", "v_proj", "o_proj"],
                        help="Target modules for LoRA")

    # Memory optimization
    parser.add_argument("--use-4bit", action="store_true", help="Use 4-bit quantization")
    parser.add_argument("--use-8bit", action="store_true", help="Use 8-bit quantization")
    parser.add_argument("--cpu", action="store_true", help="Force CPU training")
    parser.add_argument("--no-gradient-checkpointing", action="store_true",
                        help="Disable gradient checkpointing")

    # Optional
    parser.add_argument("--tools-json", help="JSON file with tool definitions for training")
    parser.add_argument("--eval-split", type=float, default=0.1, help="Eval split ratio")

    args = parser.parse_args()

    # Resolve paths
    model_path = Path(args.model)
    data_path = Path(args.data)
    output_dir = Path(args.output)

    if not model_path.exists() and "/" not in args.model:
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    if not data_path.exists():
        print(f"Error: Training data not found at {data_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Model: {model_path}")
    print(f"Data: {data_path}")
    print(f"Output: {output_dir}")

    # Load optional tools
    tools = None
    if args.tools_json:
        with open(args.tools_json) as f:
            tools = json.load(f)
        print(f"Loaded {len(tools)} tool definitions")

    # Load training data
    print("\n=== Loading training data ===")
    examples = load_training_data(data_path)

    if not examples:
        print("Error: No training examples found", file=sys.stderr)
        sys.exit(1)

    # Load tokenizer
    print("\n=== Loading tokenizer ===")
    tokenizer = AutoTokenizer.from_pretrained(args.model)

    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token
        tokenizer.pad_token_id = tokenizer.eos_token_id

    # Prepare dataset
    print("\n=== Preparing dataset ===")
    dataset = prepare_dataset(examples, tokenizer, max_length=args.max_length, tools=tools)
    print(f"Dataset size: {len(dataset)} examples")

    # Split train/eval
    if args.eval_split > 0:
        split = dataset.train_test_split(test_size=args.eval_split, seed=42)
        train_dataset = split["train"]
        eval_dataset = split["test"]
        print(f"Train: {len(train_dataset)}, Eval: {len(eval_dataset)}")
    else:
        train_dataset = dataset
        eval_dataset = None

    # Load model
    print("\n=== Loading model ===")

    if args.cpu:
        print("Using CPU (this will be slow)...")
        model = AutoModelForCausalLM.from_pretrained(
            args.model,
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
            args.model,
            quantization_config=bnb_config,
            device_map={"": 0} if torch.cuda.is_available() else {"": "cpu"},
            trust_remote_code=True,
        )
        model = prepare_model_for_kbit_training(model)
    elif args.use_8bit:
        print("Using 8-bit quantization...")
        bnb_config = BitsAndBytesConfig(load_in_8bit=True)
        model = AutoModelForCausalLM.from_pretrained(
            args.model,
            quantization_config=bnb_config,
            device_map={"": 0} if torch.cuda.is_available() else {"": "cpu"},
            trust_remote_code=True,
        )
        model = prepare_model_for_kbit_training(model)
    else:
        print("Using full precision...")
        model = AutoModelForCausalLM.from_pretrained(
            args.model,
            torch_dtype=torch.float16 if torch.cuda.is_available() else torch.float32,
            device_map={"": 0} if torch.cuda.is_available() else {"": "cpu"},
            trust_remote_code=True,
        )

    # Gradient checkpointing
    if not args.no_gradient_checkpointing:
        model.gradient_checkpointing_enable()
        print("Gradient checkpointing enabled.")

    # Configure LoRA
    print("\n=== Configuring LoRA ===")
    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=args.lora_r,
        lora_alpha=args.lora_alpha,
        lora_dropout=args.lora_dropout,
        target_modules=args.target_modules,
        bias="none",
    )

    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    # Training arguments
    print("\n=== Setting up training ===")

    effective_batch_size = 8
    gradient_accumulation = max(1, effective_batch_size // args.batch_size)

    training_args = TrainingArguments(
        output_dir=str(output_dir),
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        per_device_eval_batch_size=args.batch_size,
        learning_rate=args.learning_rate,
        weight_decay=0.01,
        logging_steps=50,
        eval_strategy="epoch" if eval_dataset else "no",
        save_strategy="epoch",
        load_best_model_at_end=eval_dataset is not None,
        metric_for_best_model="eval_loss" if eval_dataset else None,
        greater_is_better=False,
        warmup_ratio=0.1,
        lr_scheduler_type="cosine",
        fp16=torch.cuda.is_available() and not args.use_4bit and not args.cpu,
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

    # Trainer
    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=eval_dataset,
        data_collator=DataCollatorForLanguageModeling(tokenizer=tokenizer, mlm=False),
    )

    # Train
    print("\n=== Starting training ===")
    trainer.train()

    # Save
    print(f"\n=== Saving LoRA adapter to {output_dir} ===")
    model.save_pretrained(output_dir)
    tokenizer.save_pretrained(output_dir)

    print("\nTraining complete!")
    print(f"LoRA adapter saved to: {output_dir}")
    print(f"\nUsage:")
    print(f"  from peft import PeftModel")
    print(f"  model = AutoModelForCausalLM.from_pretrained('{args.model}')")
    print(f"  model = PeftModel.from_pretrained(model, '{output_dir}')")


if __name__ == "__main__":
    main()
