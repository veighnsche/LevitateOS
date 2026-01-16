#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Installing Python dependencies ==="
pip install -q -r python/requirements.txt

echo "=== Generating augmented training data ==="
python3 python/augment_data.py

echo "=== Starting LoRA training ==="
python3 python/train_lora.py \
    --model ../../vendor/models/FunctionGemma \
    --output python/adapters/installer \
    --data-dir python/training \
    --epochs 3 \
    --batch-size 1 \
    --lora-r 16 \
    --lora-alpha 32 \
    --use-4bit \
    "$@"

echo "=== Training complete ==="
echo "Adapter saved to: python/adapters/installer"
