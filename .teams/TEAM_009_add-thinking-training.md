# TEAM_009: Add Thinking to Training Data

## Status: IN PROGRESS (manual generation)

## Goal
Add `<think>` reasoning blocks to training data, then update `train_lora.py` to use them.

## Progress
- ✅ Created annotation script (`annotate_thinking.py`)
- ✅ Modified `train_lora.py` to use thinking field
- ✅ Annotated 2919/7716 examples via API
- 🔄 Remaining 4797 examples to be generated manually (API credit issue)

## Assignment
See `crates/installer/python/THINKING_ASSIGNMENT.md` for detailed instructions.

**Summary:** Generate 2-4 line reasoning for each pending example explaining:
1. What user wants
2. Relevant system context
3. Why this response is appropriate

## Files
- `crates/installer/python/annotate_thinking.py` - Annotation script
- `crates/installer/python/train_lora.py` - Training script (modified)
- `crates/installer/python/training/training_with_thinking.jsonl` - Output (2919 done)
- `crates/installer/python/training/.thinking_checkpoint.json` - Progress tracker
- `crates/installer/python/THINKING_ASSIGNMENT.md` - Assignment doc

## Next Steps
1. Generate thinking for remaining 4797 examples
2. Verify: `wc -l training/training_with_thinking.jsonl` (should be 7716)
3. Train: `python train_lora.py --epochs 1 --use-4bit`
