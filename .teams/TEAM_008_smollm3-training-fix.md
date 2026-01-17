# TEAM_008: SmolLM3 Training Fix

## Task
Remove `/no_think` prefix from training to align training format with inference/testing.

## Problem
Training was using `/no_think` prefix in system prompts while inference was not. This mismatch could cause the model to behave differently during inference than what it learned during training.

## Changes
- `crates/installer/python/train_lora.py` line 138: Update comment
- `crates/installer/python/train_lora.py` line 144: Remove `/no_think ` prefix

## Status
Complete
