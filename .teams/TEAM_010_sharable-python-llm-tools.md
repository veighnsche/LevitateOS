# TEAM_010: Sharable Python LLM Tools

## Task
Create a new `python/` folder in root with three sharable Python applications:
1. LLM server for inference
2. Training data generation
3. LoRA trainer

## Reference
Based on existing code in `crates/installer/python/`

## Status
- [x] Read existing Python code to understand patterns
- [x] Create root `python/` folder structure
- [x] Create inference server
- [x] Create training data generator
- [x] Create LoRA trainer
- [x] Create requirements.txt

## Created Files
- `python/llm_server.py` - HTTP server for LLM inference with LoRA support
- `python/generate_data.py` - Training data validation, augmentation, thinking annotation
- `python/train_lora.py` - LoRA fine-tuning with loss masking
- `python/requirements.txt` - Python dependencies

## Decisions
- Made tools generic (removed LevitateOS/installer-specific logic)
- Kept same patterns: HuggingFace transformers, PEFT, JSONL format
- Added subcommands to generate_data.py: validate, augment, annotate
- Support both standalone JSONL files and directories of JSONL files
