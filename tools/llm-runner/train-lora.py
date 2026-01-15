#!/usr/bin/env python3
"""
Train LoRA adapters for FunctionGemma.

Creates modular task-specific adapters that can be swapped at runtime.
Optimized for RTX 3060 12GB via Thunderbolt eGPU (Razer Core X).
"""

import argparse
import json
import os
import sys
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, get_peft_model, TaskType, prepare_model_for_kbit_training
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
from trl import SFTTrainer, SFTConfig


# =============================================================================
# Tool Definition (must match llm-runner.py)
# =============================================================================
SHELL_COMMAND_TOOL = {
    "type": "function",
    "function": {
        "name": "run_shell_command",
        "description": "Execute a shell command on a Linux system.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        }
    }
}

SYSTEM_PROMPT = "You are a model that can do function calling with the following functions."


# =============================================================================
# Dataset Loading
# =============================================================================
def load_training_data(path: str) -> list[dict]:
    """Load training examples from JSONL file."""
    examples = []
    with open(path) as f:
        for line in f:
            if line.strip():
                data = json.loads(line)
                examples.append({
                    "query": data["query"],
                    "command": data["command"]
                })
    return examples


def format_example(example: dict, tokenizer) -> str:
    """
    Format a training example as FunctionGemma chat with tool call.

    Returns the full conversation including the assistant's function call response.
    """
    # Build messages with tool call in assistant response
    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": example["query"]},
        {
            "role": "assistant",
            "content": None,
            "tool_calls": [{
                "id": "call_0",
                "type": "function",
                "function": {
                    "name": "run_shell_command",
                    "arguments": {"command": example["command"]}
                }
            }]
        }
    ]

    # Apply chat template with tools
    formatted = tokenizer.apply_chat_template(
        messages,
        tools=[SHELL_COMMAND_TOOL],
        tokenize=False,
        add_generation_prompt=False
    )

    return formatted


def create_dataset(examples: list[dict], tokenizer) -> Dataset:
    """Create HuggingFace Dataset from examples."""
    formatted_examples = []
    for ex in examples:
        try:
            text = format_example(ex, tokenizer)
            formatted_examples.append({"text": text})
        except Exception as e:
            print(f"Warning: Failed to format example '{ex['query']}': {e}")
            continue

    return Dataset.from_list(formatted_examples)


# =============================================================================
# Model Setup
# =============================================================================
def setup_model_and_tokenizer(model_path: str, use_4bit: bool = False):
    """Load model and tokenizer with optional quantization."""
    print(f"Loading tokenizer from {model_path}...")
    tokenizer = AutoTokenizer.from_pretrained(model_path)

    # Ensure pad token is set
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    print(f"Loading model from {model_path}...")

    if use_4bit:
        # 4-bit quantization for memory efficiency
        bnb_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.float16,
            bnb_4bit_use_double_quant=True,
        )
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            quantization_config=bnb_config,
            device_map="auto",
            trust_remote_code=True,
        )
        model = prepare_model_for_kbit_training(model)
    else:
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=torch.float16,
            device_map="auto",
            trust_remote_code=True,
        )

    # Enable gradient checkpointing
    model.gradient_checkpointing_enable()

    return model, tokenizer


def setup_lora(model, rank: int = 16, alpha: int = 32, dropout: float = 0.05):
    """Configure and apply LoRA to model."""
    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=rank,
        lora_alpha=alpha,
        lora_dropout=dropout,
        target_modules=[
            "q_proj",
            "k_proj",
            "v_proj",
            "o_proj",
            "gate_proj",
            "up_proj",
            "down_proj",
        ],
        bias="none",
    )

    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    return model


# =============================================================================
# Training
# =============================================================================
def train(
    model,
    tokenizer,
    dataset: Dataset,
    output_dir: str,
    epochs: int = 3,
    batch_size: int = 8,
    learning_rate: float = 2e-4,
):
    """Run LoRA training."""

    # Training configuration optimized for RTX 3060 eGPU
    training_args = SFTConfig(
        output_dir=output_dir,
        num_train_epochs=epochs,
        per_device_train_batch_size=batch_size,
        gradient_accumulation_steps=4,
        learning_rate=learning_rate,
        lr_scheduler_type="cosine",
        warmup_ratio=0.1,
        fp16=True,
        bf16=False,
        optim="paged_adamw_8bit",
        logging_steps=10,
        save_strategy="epoch",
        save_total_limit=2,
        # eGPU optimizations - minimize CPU<->GPU transfers
        dataloader_pin_memory=False,
        dataloader_num_workers=0,
        # Disable evaluation (we don't have a val set)
        do_eval=False,
        # Dataset config
        dataset_text_field="text",
        max_seq_length=512,
        packing=False,
    )

    trainer = SFTTrainer(
        model=model,
        args=training_args,
        train_dataset=dataset,
        processing_class=tokenizer,
    )

    print(f"\nStarting training for {epochs} epochs...")
    print(f"  Batch size: {batch_size}")
    print(f"  Learning rate: {learning_rate}")
    print(f"  Dataset size: {len(dataset)} examples")
    print()

    trainer.train()

    return trainer


# =============================================================================
# Main
# =============================================================================
def main():
    parser = argparse.ArgumentParser(
        description="Train LoRA adapters for FunctionGemma",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --base-model vendor/models/FunctionGemma --dataset training/installation_dataset.jsonl --output adapters/install.lora
  %(prog)s -m vendor/models/FunctionGemma -d training/installation_dataset.jsonl -o adapters/install.lora --epochs 5
"""
    )

    parser.add_argument(
        "--base-model", "-m",
        required=True,
        help="Path to base FunctionGemma model"
    )
    parser.add_argument(
        "--dataset", "-d",
        required=True,
        help="Path to training dataset (JSONL with query/command pairs)"
    )
    parser.add_argument(
        "--output", "-o",
        required=True,
        help="Output directory for LoRA adapter"
    )
    parser.add_argument(
        "--epochs", "-e",
        type=int,
        default=3,
        help="Number of training epochs (default: 3)"
    )
    parser.add_argument(
        "--batch-size", "-b",
        type=int,
        default=8,
        help="Training batch size (default: 8)"
    )
    parser.add_argument(
        "--learning-rate", "-lr",
        type=float,
        default=2e-4,
        help="Learning rate (default: 2e-4)"
    )
    parser.add_argument(
        "--lora-rank", "-r",
        type=int,
        default=16,
        help="LoRA rank (default: 16)"
    )
    parser.add_argument(
        "--lora-alpha", "-a",
        type=int,
        default=32,
        help="LoRA alpha (default: 32)"
    )
    parser.add_argument(
        "--use-4bit",
        action="store_true",
        help="Use 4-bit quantization (saves memory)"
    )

    args = parser.parse_args()

    # Verify GPU
    if not torch.cuda.is_available():
        print("Error: CUDA not available. This script requires a GPU.")
        sys.exit(1)

    print(f"GPU: {torch.cuda.get_device_name(0)}")
    print(f"VRAM: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f}GB")
    print()

    # Verify paths
    if not Path(args.base_model).exists():
        print(f"Error: Base model not found: {args.base_model}")
        sys.exit(1)

    if not Path(args.dataset).exists():
        print(f"Error: Dataset not found: {args.dataset}")
        sys.exit(1)

    # Load training data
    print(f"Loading training data from {args.dataset}...")
    examples = load_training_data(args.dataset)
    print(f"  Loaded {len(examples)} examples")

    # Setup model and tokenizer
    model, tokenizer = setup_model_and_tokenizer(args.base_model, args.use_4bit)

    # Create dataset
    print("Formatting dataset...")
    dataset = create_dataset(examples, tokenizer)
    print(f"  Created dataset with {len(dataset)} formatted examples")

    # Setup LoRA
    print(f"Configuring LoRA (rank={args.lora_rank}, alpha={args.lora_alpha})...")
    model = setup_lora(model, args.lora_rank, args.lora_alpha)

    # Train
    trainer = train(
        model=model,
        tokenizer=tokenizer,
        dataset=dataset,
        output_dir=args.output,
        epochs=args.epochs,
        batch_size=args.batch_size,
        learning_rate=args.learning_rate,
    )

    # Save final adapter
    print(f"\nSaving adapter to {args.output}...")
    trainer.save_model(args.output)
    tokenizer.save_pretrained(args.output)

    # Report size
    adapter_size = sum(
        f.stat().st_size for f in Path(args.output).rglob("*") if f.is_file()
    ) / 1e6
    print(f"  Adapter size: {adapter_size:.1f}MB")

    print("\nTraining complete!")
    print(f"To use: python3 llm-runner.py --model {args.base_model} --adapter {args.output} -p 'your query'")


if __name__ == "__main__":
    main()
