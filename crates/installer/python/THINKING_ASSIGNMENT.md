# Assignment: Generate Thinking Content for Training Data

## Goal
Add reasoning/thinking content to 4797 remaining training examples for the LevitateOS installer LLM.

## Current State
- **Total examples:** 7716
- **Already annotated:** 2919 (in `training/training_with_thinking.jsonl`)
- **Remaining:** 4797

## Task
For each pending example, generate 2-4 lines of internal reasoning that explains:
1. What the user wants or needs
2. Relevant system context (boot mode, disk, current state)
3. Why this specific response or command is appropriate

## Input Format
Each example has:
```json
{
  "system_context": "## Current System State\n- Boot mode: UEFI\n- Network: Connected\n...",
  "messages": [
    {"role": "user", "content": "partition the disk"}
  ],
  "expected_response": {
    "type": "command",  // or "text"
    "command": "sgdisk -Z /dev/sda && ..."  // if type=command
    "response": "..."  // if type=text
  }
}
```

## Output Format
Same as input, but with `thinking` field added to `expected_response`:
```json
{
  "system_context": "...",
  "messages": [...],
  "expected_response": {
    "type": "command",
    "command": "sgdisk -Z /dev/sda && ...",
    "thinking": "User wants to partition disk for Linux install. UEFI system needs EFI partition (512M, ef00) and root partition. Using sgdisk for GPT partitioning."
  }
}
```

## Reasoning Style Guide
- **Concise:** 2-4 lines max
- **Internal:** This is the model's internal reasoning, not user-facing
- **Contextual:** Reference specific details from system_context (boot mode, disk names)
- **Purposeful:** Explain WHY this response/command is appropriate

### Example Reasonings

**For commands:**
```
User wants to list available disks. Using lsblk to show block devices with size info. This helps user identify which disk to install on.
```

**For text responses:**
```
User expressing anxiety about installation. Acknowledging concern builds trust before proceeding with potentially destructive disk operations. Asking for specifics helps provide targeted reassurance.
```

**For confirmations:**
```
User confirmed partition action. System has /dev/sda (500G). Executing sgdisk to create GPT layout: 512M EFI partition (ef00) and remainder for root.
```

## Process
1. Read batch of pending examples from source files
2. Generate thinking for each
3. Append annotated examples to `training/training_with_thinking.jsonl`
4. Update checkpoint to track progress
5. Repeat until all 4797 are done

## Files
- **Source:** `training/augmented_dataset.jsonl`, `training/targeted_weaknesses.jsonl`
- **Output:** `training/training_with_thinking.jsonl` (append)
- **Checkpoint:** `training/.thinking_checkpoint.json`

## Verification
After completion:
```bash
wc -l training/training_with_thinking.jsonl  # Should be 7716
python train_lora.py --epochs 1 --use-4bit   # Train with new data
```
