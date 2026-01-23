# TEAM_022: LLM Toolkit Extraction

## Goal
Complete the extraction of generic LLM training tools from `installer/python/` to `llm-toolkit/`, then make installer use the toolkit.

## Status: COMPLETED

## Changes Made

### llm-toolkit/ (Generic Toolkit)

1. **train_lora.py** - Enhanced with:
   - Full target module support (attention + MLP layers)
   - Tool definitions via `--tools-json`
   - Flexible hyperparameters with sensible defaults

2. **llm_server.py** - Rewritten as extensible base class:
   - `LLMServer` class with override hooks:
     - `gather_context()` - Dynamic context injection
     - `build_system_prompt()` - Custom prompt building
     - `verify_response()` - Response validation/filtering
   - Support for both `/generate` and `/query` endpoints
   - Generic tool/function call parsing (XML and SmolLM3 styles)
   - `run_server()` helper for custom server instances

3. **evaluate.py** - Enhanced with:
   - Better tool call extraction (XML, SmolLM3, bash blocks, $ prefix)
   - Configurable system prompt and tool schema
   - Sweep directory support

4. **__init__.py** - Created for package imports:
   - Exports: `LLMServer`, `run_server`, `ModelEvaluator`, `EvalResult`, `EvalSummary`

### installer/python/ (Now Uses Toolkit)

1. **llm_server.py** - Now a thin wrapper:
   - `InstallerLLMServer(LLMServer)` subclass
   - Overrides `gather_context()` for system facts
   - Overrides `build_system_prompt()` for installer template
   - Overrides `verify_response()` for disk hallucination detection
   - Reduced from 410 lines to 317 lines

2. **evaluate_lora.py** - Now uses toolkit:
   - `InstallerEvaluator(ModelEvaluator)` subclass
   - Imports from llm-toolkit/evaluate.py
   - Added `--export-tests` to export test cases to JSONL
   - Reduced from 568 lines to 398 lines

3. **train_lora.py** - Now a thin wrapper:
   - Calls llm-toolkit/train_lora.py with installer defaults
   - Creates tools.json automatically
   - Installer-optimized hyperparameters (r=32, alpha=64)
   - Reduced from 507 lines to 161 lines

### Data Generation (Unchanged)

- `augment_data.py` - Kept as-is (heavily installer-specific)
  - Disk configs, boot modes, partition naming
  - System state tracking, template expansion
  - This is installer-domain code, not generic

## Architecture

```
llm-toolkit/                    # Generic LLM tools
  __init__.py                   # Package exports
  train_lora.py                 # Generic LoRA training
  llm_server.py                 # Extensible HTTP server
  evaluate.py                   # Generic evaluation
  generate_data.py              # Generic data utilities

installer/python/               # Installer-specific
  llm_server.py                 # InstallerLLMServer extends LLMServer
  evaluate_lora.py              # InstallerEvaluator extends ModelEvaluator
  train_lora.py                 # Wrapper with installer defaults
  augment_data.py               # Installer-specific data generation
  conversations/                # Template conversations
  training/                     # Generated training data
  adapters/                     # Trained LoRA weights
```

## Benefits

1. **No more code duplication** - Core ML logic in one place
2. **Installer uses toolkit** - Thin wrappers with customizations
3. **Extensible design** - Other domains can subclass LLMServer
4. **Same interface** - All installer scripts use the same API
