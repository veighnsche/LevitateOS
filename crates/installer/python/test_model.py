#!/usr/bin/env python3
"""
Test the trained LoRA adapter on held-out test data.

Measures:
- Command accuracy: Does the model generate the correct command?
- Response type accuracy: Does it correctly choose text vs command?
- Text response quality: BLEU/similarity for text responses

Usage:
    python test_model.py --model vendor/models/SmolLM3-3B --adapter adapters/installer
"""

import argparse
import json
import re
import sys
from pathlib import Path
from collections import defaultdict

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

# Script directory for resolving relative paths
SCRIPT_DIR = Path(__file__).parent.resolve()


# Same system prompt and tool as training
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


def load_model(model_path: Path, adapter_path: Path = None, use_4bit: bool = False):
    """Load the base model and optionally apply LoRA adapter."""
    print(f"Loading tokenizer from {model_path}...")
    tokenizer = AutoTokenizer.from_pretrained(model_path)

    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    print(f"Loading model from {model_path}...")
    if use_4bit:
        from transformers import BitsAndBytesConfig
        bnb_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.float16,
        )
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            quantization_config=bnb_config,
            device_map="auto",
            trust_remote_code=True,
        )
    else:
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=torch.float16 if torch.cuda.is_available() else torch.float32,
            device_map="auto" if torch.cuda.is_available() else None,
            trust_remote_code=True,
        )

    if adapter_path and adapter_path.exists():
        print(f"Loading LoRA adapter from {adapter_path}...")
        model = PeftModel.from_pretrained(model, adapter_path)
        model = model.merge_and_unload()  # Merge for faster inference

    model.eval()
    return model, tokenizer


def generate_response(model, tokenizer, system_context: str, messages: list, max_new_tokens: int = 256):
    """Generate a response for the given conversation."""
    system_prompt = SYSTEM_PROMPT_TEMPLATE.format(system_context=system_context)

    chat_messages = [{"role": "system", "content": system_prompt}]
    chat_messages.extend(messages)

    try:
        prompt = tokenizer.apply_chat_template(
            chat_messages,
            tools=[SHELL_COMMAND_TOOL],
            tokenize=False,
            add_generation_prompt=True
        )
    except Exception:
        # Fallback
        parts = [f"<system>{system_prompt}</system>"]
        for msg in messages:
            parts.append(f"<{msg['role']}>{msg['content']}</{msg['role']}>")
        parts.append("<assistant>")
        prompt = "\n".join(parts)

    inputs = tokenizer(prompt, return_tensors="pt")
    if torch.cuda.is_available():
        inputs = {k: v.cuda() for k, v in inputs.items()}

    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=max_new_tokens,
            do_sample=False,  # Deterministic for testing
            pad_token_id=tokenizer.pad_token_id,
            eos_token_id=tokenizer.eos_token_id,
        )

    response = tokenizer.decode(outputs[0][inputs["input_ids"].shape[1]:], skip_special_tokens=False)
    return response


def parse_response(response: str) -> dict:
    """Parse the model's response to extract type and content."""
    # Check for SmolLM3 XML-style tool call: <tool_call>{"name": ..., "arguments": ...}</tool_call>
    tool_call_match = re.search(r'<tool_call>\s*(\{[^}]+\})\s*</tool_call>', response, re.DOTALL)
    if tool_call_match:
        try:
            tool_data = json.loads(tool_call_match.group(1))
            if tool_data.get("name") == "run_shell_command":
                cmd = tool_data.get("arguments", {}).get("command", "")
                return {"type": "command", "command": cmd.strip()}
        except json.JSONDecodeError:
            pass

    # Also try simpler patterns for partial matches
    func_match = re.search(r'"command"\s*:\s*"([^"]+)"', response)
    if func_match:
        return {"type": "command", "command": func_match.group(1).strip()}

    # Otherwise it's a text response
    # Clean up any special tokens
    text = re.sub(r'<[^>]+>', '', response).strip()
    text = re.sub(r'<\|im_end\|>', '', text).strip()
    return {"type": "text", "response": text}


def normalize_command(cmd: str) -> str:
    """Normalize a command for comparison."""
    # Remove extra whitespace
    cmd = ' '.join(cmd.split())
    # Remove trailing semicolons
    cmd = cmd.rstrip(';')
    return cmd


def evaluate_test_set(model, tokenizer, test_file: Path, max_examples: int = None, verbose: bool = False):
    """Evaluate the model on the test set."""
    results = {
        "total": 0,
        "type_correct": 0,
        "command_exact_match": 0,
        "command_partial_match": 0,
        "text_responses": 0,
        "by_type": defaultdict(lambda: {"total": 0, "correct": 0}),
    }

    examples = []
    with open(test_file) as f:
        for line in f:
            if line.strip():
                examples.append(json.loads(line))

    if max_examples:
        examples = examples[:max_examples]

    print(f"\nEvaluating on {len(examples)} test examples...")

    for i, example in enumerate(examples):
        system_context = example["system_context"]
        messages = example["messages"]
        expected = example["expected_response"]

        # Generate response
        response = generate_response(model, tokenizer, system_context, messages)
        parsed = parse_response(response)

        results["total"] += 1

        # Check type accuracy
        type_correct = parsed["type"] == expected["type"]
        if type_correct:
            results["type_correct"] += 1

        results["by_type"][expected["type"]]["total"] += 1

        # Check content accuracy
        if expected["type"] == "command":
            expected_cmd = normalize_command(expected["command"])
            actual_cmd = normalize_command(parsed.get("command", ""))

            if expected_cmd == actual_cmd:
                results["command_exact_match"] += 1
                results["by_type"]["command"]["correct"] += 1
            elif expected_cmd in actual_cmd or actual_cmd in expected_cmd:
                results["command_partial_match"] += 1
        else:
            results["text_responses"] += 1
            if type_correct:
                results["by_type"]["text"]["correct"] += 1

        if verbose or (i + 1) % 50 == 0:
            print(f"  [{i+1}/{len(examples)}] Type acc: {100*results['type_correct']/results['total']:.1f}%")

        if verbose:
            print(f"\n--- Example {i+1} ---")
            print(f"User: {messages[-1]['content'][:50]}...")
            print(f"Expected: {expected['type']} - {expected.get('command', expected.get('response', ''))[:50]}...")
            print(f"Got: {parsed['type']} - {parsed.get('command', parsed.get('response', ''))[:50]}...")
            print(f"Type correct: {type_correct}")

    return results


def print_results(results: dict):
    """Print evaluation results."""
    print("\n" + "="*60)
    print("EVALUATION RESULTS")
    print("="*60)

    print(f"\nTotal examples: {results['total']}")
    print(f"\nResponse Type Accuracy: {100*results['type_correct']/results['total']:.1f}%")

    print(f"\nCommand Generation:")
    cmd_total = results["by_type"]["command"]["total"]
    if cmd_total > 0:
        print(f"  Exact match: {results['command_exact_match']}/{cmd_total} ({100*results['command_exact_match']/cmd_total:.1f}%)")
        print(f"  Partial match: {results['command_partial_match']}/{cmd_total} ({100*results['command_partial_match']/cmd_total:.1f}%)")

    print(f"\nText Responses:")
    text_total = results["by_type"]["text"]["total"]
    if text_total > 0:
        text_correct = results["by_type"]["text"]["correct"]
        print(f"  Type correct: {text_correct}/{text_total} ({100*text_correct/text_total:.1f}%)")

    print("\n" + "="*60)


def main():
    # Default test file relative to script location
    default_test_file = SCRIPT_DIR / "testing" / "test_dataset.jsonl"

    parser = argparse.ArgumentParser(description="Test trained LoRA adapter")
    parser.add_argument("--model", "-m", default="vendor/models/SmolLM3-3B",
                        help="Base model path")
    parser.add_argument("--adapter", "-a", default=None,
                        help="LoRA adapter path (optional, tests base model if not provided)")
    parser.add_argument("--test-file", "-t", default=str(default_test_file),
                        help="Test dataset file")
    parser.add_argument("--max-examples", "-n", type=int, default=100,
                        help="Maximum examples to test (default 100, use -1 for all)")
    parser.add_argument("--use-4bit", action="store_true",
                        help="Use 4-bit quantization")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show each example")
    args = parser.parse_args()

    # Resolve model path
    model_path = Path(args.model)
    if not model_path.is_absolute() and not model_path.exists():
        # Try relative to project root (script dir's parent x3)
        project_root = SCRIPT_DIR.parent.parent.parent
        model_path = project_root / args.model
    model_path = model_path.resolve()

    # Resolve adapter path if provided
    adapter_path = None
    if args.adapter:
        adapter_path = Path(args.adapter)
        if not adapter_path.is_absolute() and not adapter_path.exists():
            adapter_path = SCRIPT_DIR / args.adapter
        adapter_path = adapter_path.resolve()

    # Resolve test file path
    test_file = Path(args.test_file).resolve()

    if not model_path.exists():
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    if not test_file.exists():
        print(f"Error: Test file not found at {test_file}", file=sys.stderr)
        print("Run augment_data.py first to generate train/test split.", file=sys.stderr)
        sys.exit(1)

    # Load model
    model, tokenizer = load_model(model_path, adapter_path, args.use_4bit)

    # Evaluate
    max_examples = None if args.max_examples == -1 else args.max_examples
    results = evaluate_test_set(model, tokenizer, test_file, max_examples, args.verbose)

    # Print results
    print_results(results)


if __name__ == "__main__":
    main()
