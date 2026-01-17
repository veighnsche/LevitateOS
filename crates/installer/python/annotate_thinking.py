#!/usr/bin/env python3
"""
Add thinking/reasoning to training data using Claude Haiku 4.5.

Supports both sync API and batch API (50% savings).

Usage:
    # Sync mode (realtime, resumes from checkpoint)
    python annotate_thinking.py

    # Batch mode (50% savings, async processing)
    python annotate_thinking.py --batch           # Submit batch
    python annotate_thinking.py --batch --status  # Check status
    python annotate_thinking.py --batch --process # Download results
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path

try:
    import anthropic
except ImportError:
    print("Error: pip install anthropic", file=sys.stderr)
    sys.exit(1)

try:
    from dotenv import load_dotenv
except ImportError:
    print("Error: pip install python-dotenv", file=sys.stderr)
    sys.exit(1)

SCRIPT_DIR = Path(__file__).parent.resolve()
load_dotenv(SCRIPT_DIR / ".env")

ONE_SHOT_EXAMPLE = """Example:
System: UEFI boot, /dev/sda (500G Samsung SSD)
User: "partition the disk"
Response: command - sgdisk -Z /dev/sda && sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 /dev/sda

Reasoning:
User wants to partition for Linux install. UEFI needs EFI partition (512M, type ef00) and root partition. Using sgdisk for GPT. -Z wipes existing partitions first."""

THINKING_PROMPT = """Generate 2-4 lines of internal reasoning for this OS installer response.

{one_shot}

Now generate reasoning for:
System context: {system_context}
Conversation: {conversation}
Response type: {response_type}
Response: {response_content}

Reasoning (2-4 lines, concise):"""


def format_conversation(messages):
    lines = []
    for msg in messages:
        role = "User" if msg["role"] == "user" else "Assistant"
        lines.append(f'{role}: "{msg["content"]}"')
    return "\n".join(lines)


def get_response_content(expected):
    if expected["type"] == "command":
        return expected["command"]
    return expected.get("response", "")


def compress_context(system_context):
    if len(system_context) > 200:
        lines = system_context.split('\n')
        key_lines = [l for l in lines if any(k in l.lower() for k in ['boot', 'disk', '/dev', 'network', 'hostname'])]
        return '; '.join(l.strip('- ').strip() for l in key_lines[:4])
    return system_context


def build_prompt(example):
    system_context = compress_context(example.get("system_context", "UEFI boot, standard disk setup"))
    conversation = format_conversation(example["messages"])
    response_type = example["expected_response"]["type"]
    response_content = get_response_content(example["expected_response"])
    return THINKING_PROMPT.format(
        one_shot=ONE_SHOT_EXAMPLE,
        system_context=system_context,
        conversation=conversation,
        response_type=response_type,
        response_content=response_content
    )


def load_checkpoint(path):
    if path.exists():
        return json.loads(path.read_text())
    return {"processed": [], "success": 0, "errors": 0}


def save_checkpoint(path, data):
    path.write_text(json.dumps(data))


def load_all_examples(data_dir):
    examples = []
    for f in sorted(data_dir.glob("*.jsonl")):
        if "_with_thinking" in f.name:
            continue
        for i, line in enumerate(f.read_text().strip().split('\n'), 1):
            if not line:
                continue
            try:
                ex = json.loads(line)
                if "messages" in ex and "expected_response" in ex:
                    examples.append((f.name, i, ex))
            except:
                continue
    return examples


# ==================== SYNC MODE ====================

def run_sync(client, examples, checkpoint_path, output_file, model, delay):
    checkpoint = load_checkpoint(checkpoint_path)
    processed_set = set(checkpoint["processed"])

    if processed_set:
        print(f"Resuming: {len(processed_set)} done, {checkpoint['success']} success")

    total = len(examples)
    print(f"\nProcessing {total} examples (sync mode, {delay}s delay)...\n")

    with open(output_file, "a") as out:
        for idx, (fname, line, ex) in enumerate(examples):
            if idx in processed_set:
                continue

            user_msg = ex["messages"][-1]["content"][:35]
            resp_type = ex["expected_response"]["type"]
            print(f"[{idx+1}/{total}] {resp_type}: \"{user_msg}...\"", end=" ", flush=True)

            try:
                prompt = build_prompt(ex)
                msg = client.messages.create(
                    model=model,
                    max_tokens=150,
                    messages=[{"role": "user", "content": prompt}]
                )
                thinking = msg.content[0].text.strip()
                print("OK")
                print(f"    Think: {thinking[:70]}...")

                annotated = ex.copy()
                annotated["expected_response"] = ex["expected_response"].copy()
                annotated["expected_response"]["thinking"] = thinking
                out.write(json.dumps(annotated) + "\n")
                out.flush()

                checkpoint["success"] += 1
                checkpoint["processed"].append(idx)
                time.sleep(delay)

            except anthropic.RateLimitError:
                print("RATE LIMIT - wait 60s")
                time.sleep(60)
                continue
            except Exception as e:
                print(f"ERR: {e}")
                checkpoint["errors"] += 1
                checkpoint["processed"].append(idx)

            if len(checkpoint["processed"]) % 100 == 0:
                save_checkpoint(checkpoint_path, checkpoint)

    save_checkpoint(checkpoint_path, checkpoint)
    print(f"\nDone! Success: {checkpoint['success']}, Errors: {checkpoint['errors']}")


# ==================== BATCH MODE ====================

def get_pending_indices(examples, checkpoint_path):
    """Get indices of examples not yet processed."""
    checkpoint = load_checkpoint(checkpoint_path)
    processed_set = set(checkpoint["processed"])
    return [i for i in range(len(examples)) if i not in processed_set]


def submit_batch(client, examples, pending_indices, model, state_file):
    print(f"Building batch for {len(pending_indices)} pending examples...")

    requests = []
    for idx in pending_indices:
        fname, line, ex = examples[idx]
        prompt = build_prompt(ex)
        requests.append({
            "custom_id": str(idx),
            "params": {
                "model": model,
                "max_tokens": 150,
                "messages": [{"role": "user", "content": prompt}]
            }
        })

    print(f"Submitting batch of {len(requests)} requests...")
    batch = client.messages.batches.create(requests=requests)

    state = {
        "batch_id": batch.id,
        "num_requests": len(requests),
        "status": batch.processing_status,
        "indices": pending_indices,
    }
    state_file.write_text(json.dumps(state, indent=2))

    print(f"\nBatch submitted!")
    print(f"  ID: {batch.id}")
    print(f"  Requests: {len(requests)}")
    print(f"  Status: {batch.processing_status}")
    print(f"\nCheck with: python annotate_thinking.py --batch --status")


def check_batch_status(client, state_file):
    if not state_file.exists():
        print("No batch submitted. Run with --batch first.")
        return

    state = json.loads(state_file.read_text())
    batch = client.messages.batches.retrieve(state["batch_id"])

    print(f"Batch ID: {batch.id}")
    print(f"Status: {batch.processing_status}")
    print(f"Succeeded: {batch.request_counts.succeeded}/{state['num_requests']}")
    print(f"Errored: {batch.request_counts.errored}")

    if batch.processing_status == "ended":
        print("\nBatch complete! Run with --batch --process to download results.")


def process_batch_results(client, examples, state_file, checkpoint_path, output_file):
    if not state_file.exists():
        print("No batch submitted.")
        return

    state = json.loads(state_file.read_text())
    batch = client.messages.batches.retrieve(state["batch_id"])

    if batch.processing_status != "ended":
        print(f"Batch not done. Status: {batch.processing_status}")
        return

    print(f"Downloading results...")

    checkpoint = load_checkpoint(checkpoint_path)
    success = 0
    errors = 0

    with open(output_file, "a") as out:
        for entry in client.messages.batches.results(state["batch_id"]):
            idx = int(entry.custom_id)

            if entry.result.type == "succeeded":
                thinking = entry.result.message.content[0].text.strip()
                fname, line, ex = examples[idx]

                annotated = ex.copy()
                annotated["expected_response"] = ex["expected_response"].copy()
                annotated["expected_response"]["thinking"] = thinking
                out.write(json.dumps(annotated) + "\n")

                checkpoint["processed"].append(idx)
                checkpoint["success"] += 1
                success += 1
            else:
                checkpoint["processed"].append(idx)
                checkpoint["errors"] += 1
                errors += 1

    save_checkpoint(checkpoint_path, checkpoint)
    print(f"\nDone! This batch: {success} success, {errors} errors")
    print(f"Total: {checkpoint['success']} success, {checkpoint['errors']} errors")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data-dir", "-d", default=str(SCRIPT_DIR / "training"))
    parser.add_argument("--model", "-m", default="claude-haiku-4-5")
    parser.add_argument("--delay", type=float, default=1.3, help="Sync mode delay (seconds)")
    parser.add_argument("--batch", action="store_true", help="Use batch API (50% savings)")
    parser.add_argument("--status", action="store_true", help="Check batch status")
    parser.add_argument("--process", action="store_true", help="Process batch results")
    args = parser.parse_args()

    data_dir = Path(args.data_dir).resolve()
    checkpoint_path = data_dir / ".thinking_checkpoint.json"
    output_file = data_dir / "training_with_thinking.jsonl"
    batch_state_file = data_dir / ".batch_state.json"

    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY not set")
        sys.exit(1)

    client = anthropic.Anthropic(api_key=api_key)

    print(f"Loading examples from {data_dir}...")
    examples = load_all_examples(data_dir)
    print(f"Found {len(examples)} total examples")

    if args.batch:
        if args.status:
            check_batch_status(client, batch_state_file)
        elif args.process:
            process_batch_results(client, examples, batch_state_file, checkpoint_path, output_file)
        else:
            pending = get_pending_indices(examples, checkpoint_path)
            print(f"Pending: {len(pending)} examples")
            if pending:
                submit_batch(client, examples, pending, args.model, batch_state_file)
            else:
                print("All examples already processed!")
    else:
        run_sync(client, examples, checkpoint_path, output_file, args.model, args.delay)


if __name__ == "__main__":
    main()
