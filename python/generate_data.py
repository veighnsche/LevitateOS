#!/usr/bin/env python3
"""
Training Data Generator - Create and augment training data for LLM fine-tuning.

Supports multiple data generation strategies:
1. Template expansion with variable substitution
2. Conversation augmentation
3. Adding reasoning/thinking annotations via Claude API

Input format (JSONL):
{
    "messages": [
        {"role": "user", "content": "Hello"},
        {"role": "assistant", "content": "Hi there!"}
    ],
    "expected_response": {"type": "text", "response": "..."}  // for training
}

Usage:
    # Augment existing data with variations
    python generate_data.py augment --input data.jsonl --output augmented.jsonl

    # Add thinking annotations using Claude
    python generate_data.py annotate --input data.jsonl --output with_thinking.jsonl

    # Validate training data format
    python generate_data.py validate --input data.jsonl
"""

import argparse
import json
import os
import random
import sys
import time
from pathlib import Path
from typing import Optional

# For thinking annotation
try:
    import anthropic
    ANTHROPIC_AVAILABLE = True
except ImportError:
    ANTHROPIC_AVAILABLE = False

try:
    from dotenv import load_dotenv
    DOTENV_AVAILABLE = True
except ImportError:
    DOTENV_AVAILABLE = False


def load_jsonl(path: Path) -> list[dict]:
    """Load examples from a JSONL file."""
    examples = []
    with open(path) as f:
        for line_num, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                examples.append(json.loads(line))
            except json.JSONDecodeError as e:
                print(f"Warning: Invalid JSON at line {line_num}: {e}", file=sys.stderr)
    return examples


def save_jsonl(examples: list[dict], path: Path):
    """Save examples to a JSONL file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w") as f:
        for ex in examples:
            f.write(json.dumps(ex) + "\n")


# =============================================================================
# Validation
# =============================================================================

def validate_example(example: dict, idx: int) -> list[str]:
    """Validate a single training example. Returns list of errors."""
    errors = []

    if "messages" not in example:
        errors.append(f"Example {idx}: missing 'messages' field")
    elif not isinstance(example["messages"], list):
        errors.append(f"Example {idx}: 'messages' must be a list")
    elif len(example["messages"]) == 0:
        errors.append(f"Example {idx}: 'messages' is empty")
    else:
        for i, msg in enumerate(example["messages"]):
            if "role" not in msg:
                errors.append(f"Example {idx}, message {i}: missing 'role'")
            elif msg["role"] not in ["user", "assistant", "system"]:
                errors.append(f"Example {idx}, message {i}: invalid role '{msg['role']}'")
            if "content" not in msg:
                errors.append(f"Example {idx}, message {i}: missing 'content'")

    if "expected_response" in example:
        resp = example["expected_response"]
        if "type" not in resp:
            errors.append(f"Example {idx}: expected_response missing 'type'")
        elif resp["type"] == "command" and "command" not in resp:
            errors.append(f"Example {idx}: command type but no 'command' field")
        elif resp["type"] == "text" and "response" not in resp:
            errors.append(f"Example {idx}: text type but no 'response' field")

    return errors


def cmd_validate(args):
    """Validate training data format."""
    input_path = Path(args.input)
    if not input_path.exists():
        print(f"Error: File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    examples = load_jsonl(input_path)
    print(f"Loaded {len(examples)} examples from {input_path}")

    all_errors = []
    for idx, ex in enumerate(examples):
        errors = validate_example(ex, idx)
        all_errors.extend(errors)

    if all_errors:
        print(f"\nFound {len(all_errors)} errors:")
        for err in all_errors[:20]:
            print(f"  - {err}")
        if len(all_errors) > 20:
            print(f"  ... and {len(all_errors) - 20} more")
        sys.exit(1)
    else:
        print("All examples valid!")

        # Stats
        with_expected = sum(1 for ex in examples if "expected_response" in ex)
        with_thinking = sum(1 for ex in examples
                          if "expected_response" in ex
                          and "thinking" in ex.get("expected_response", {}))

        print(f"\nStats:")
        print(f"  Total examples: {len(examples)}")
        print(f"  With expected_response: {with_expected}")
        print(f"  With thinking: {with_thinking}")


# =============================================================================
# Augmentation
# =============================================================================

# Default augmentation substitutions
DEFAULT_SUBSTITUTIONS = {
    "greetings": ["hi", "hello", "hey", "good morning", "good afternoon"],
    "confirmations": ["yes", "ok", "sure", "proceed", "do it", "go ahead"],
    "negations": ["no", "cancel", "stop", "nevermind", "wait"],
}


def augment_text(text: str, substitutions: dict) -> list[str]:
    """Generate variations of text using substitutions."""
    results = [text]

    # Case variations
    results.append(text.lower())
    results.append(text.upper())
    results.append(text.capitalize())

    # Remove duplicates while preserving order
    seen = set()
    unique = []
    for r in results:
        if r not in seen:
            seen.add(r)
            unique.append(r)

    return unique


def cmd_augment(args):
    """Augment training data with variations."""
    input_path = Path(args.input)
    output_path = Path(args.output)

    if not input_path.exists():
        print(f"Error: File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    examples = load_jsonl(input_path)
    print(f"Loaded {len(examples)} examples")

    # Load custom substitutions if provided
    substitutions = DEFAULT_SUBSTITUTIONS.copy()
    if args.substitutions:
        with open(args.substitutions) as f:
            custom = json.load(f)
            substitutions.update(custom)

    augmented = []
    for ex in examples:
        augmented.append(ex)  # Keep original

        if args.case_variations and "messages" in ex:
            # Create case variations of the last user message
            messages = ex["messages"]
            if messages and messages[-1]["role"] == "user":
                original_text = messages[-1]["content"]
                variations = augment_text(original_text, substitutions)

                for variant in variations[1:]:  # Skip original
                    new_ex = ex.copy()
                    new_ex["messages"] = messages[:-1] + [
                        {"role": "user", "content": variant}
                    ]
                    augmented.append(new_ex)

    random.seed(args.seed)
    random.shuffle(augmented)

    save_jsonl(augmented, output_path)
    print(f"Saved {len(augmented)} examples to {output_path}")
    print(f"  Original: {len(examples)}")
    print(f"  Augmented: {len(augmented) - len(examples)}")


# =============================================================================
# Thinking Annotation
# =============================================================================

THINKING_PROMPT = """Generate 2-4 lines of internal reasoning for this response.

Context: {context}
Conversation: {conversation}
Response type: {response_type}
Response: {response_content}

Write concise reasoning (2-4 lines) explaining WHY this is the right response:"""


def format_conversation(messages: list[dict]) -> str:
    """Format messages for display."""
    lines = []
    for msg in messages:
        role = "User" if msg["role"] == "user" else "Assistant"
        content = msg["content"][:100] + "..." if len(msg["content"]) > 100 else msg["content"]
        lines.append(f'{role}: "{content}"')
    return "\n".join(lines)


def generate_thinking(client, example: dict, model: str) -> str:
    """Generate thinking annotation for an example."""
    context = example.get("system_context", "General assistant context")
    if len(context) > 300:
        context = context[:300] + "..."

    conversation = format_conversation(example.get("messages", []))
    expected = example.get("expected_response", {})
    response_type = expected.get("type", "text")

    if response_type == "command":
        response_content = expected.get("command", "")
    else:
        response_content = expected.get("response", "")

    prompt = THINKING_PROMPT.format(
        context=context,
        conversation=conversation,
        response_type=response_type,
        response_content=response_content
    )

    message = client.messages.create(
        model=model,
        max_tokens=150,
        messages=[{"role": "user", "content": prompt}]
    )

    return message.content[0].text.strip()


def cmd_annotate(args):
    """Add thinking annotations using Claude API."""
    if not ANTHROPIC_AVAILABLE:
        print("Error: pip install anthropic", file=sys.stderr)
        sys.exit(1)

    if DOTENV_AVAILABLE:
        load_dotenv()

    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY not set", file=sys.stderr)
        sys.exit(1)

    input_path = Path(args.input)
    output_path = Path(args.output)

    if not input_path.exists():
        print(f"Error: File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    examples = load_jsonl(input_path)
    print(f"Loaded {len(examples)} examples")

    # Filter to examples with expected_response but no thinking
    to_annotate = [
        (i, ex) for i, ex in enumerate(examples)
        if "expected_response" in ex
        and "thinking" not in ex.get("expected_response", {})
    ]

    if args.limit:
        to_annotate = to_annotate[:args.limit]

    print(f"Annotating {len(to_annotate)} examples")
    print(f"Model: {args.model}")
    print(f"Delay: {args.delay}s between requests")

    if args.dry_run:
        print("(dry run - no API calls)")
        return

    client = anthropic.Anthropic(api_key=api_key)
    annotated = []
    success = 0
    errors = 0

    for idx, (orig_idx, example) in enumerate(to_annotate):
        user_msg = example["messages"][-1]["content"][:40] if example.get("messages") else ""
        print(f"[{idx+1}/{len(to_annotate)}] \"{user_msg}...\"", end=" ", flush=True)

        try:
            thinking = generate_thinking(client, example, args.model)
            print("OK")

            new_ex = example.copy()
            new_ex["expected_response"] = example["expected_response"].copy()
            new_ex["expected_response"]["thinking"] = thinking
            annotated.append(new_ex)
            success += 1

            time.sleep(args.delay)

        except anthropic.RateLimitError:
            print("RATE LIMIT - waiting 60s")
            time.sleep(60)
            continue
        except Exception as e:
            print(f"ERROR: {e}")
            errors += 1
            annotated.append(example)  # Keep original without thinking

    save_jsonl(annotated, output_path)
    print(f"\nDone: {success} annotated, {errors} errors")
    print(f"Saved to {output_path}")


# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="Training Data Generator")
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # Validate
    p_validate = subparsers.add_parser("validate", help="Validate training data format")
    p_validate.add_argument("--input", "-i", required=True, help="Input JSONL file")

    # Augment
    p_augment = subparsers.add_parser("augment", help="Augment training data")
    p_augment.add_argument("--input", "-i", required=True, help="Input JSONL file")
    p_augment.add_argument("--output", "-o", required=True, help="Output JSONL file")
    p_augment.add_argument("--substitutions", "-s", help="JSON file with substitution mappings")
    p_augment.add_argument("--case-variations", action="store_true", help="Add case variations")
    p_augment.add_argument("--seed", type=int, default=42, help="Random seed")

    # Annotate
    p_annotate = subparsers.add_parser("annotate", help="Add thinking annotations via Claude")
    p_annotate.add_argument("--input", "-i", required=True, help="Input JSONL file")
    p_annotate.add_argument("--output", "-o", required=True, help="Output JSONL file")
    p_annotate.add_argument("--model", "-m", default="claude-haiku-4-5", help="Claude model")
    p_annotate.add_argument("--limit", "-l", type=int, help="Limit number of examples")
    p_annotate.add_argument("--delay", type=float, default=1.3, help="Delay between API calls")
    p_annotate.add_argument("--dry-run", action="store_true", help="Don't make API calls")

    args = parser.parse_args()

    if args.command == "validate":
        cmd_validate(args)
    elif args.command == "augment":
        cmd_augment(args)
    elif args.command == "annotate":
        cmd_annotate(args)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
