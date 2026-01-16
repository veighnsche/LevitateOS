#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Default preset
PRESET="${1:-default}"

echo "=== Installing Python dependencies ==="
pip install -q -r python/requirements.txt

echo "=== Generating augmented training data ==="
python3 python/augment_data.py

echo ""
echo "=========================================="
echo "  LoRA Hyperparameter Sweep"
echo "  Preset: $PRESET"
echo "=========================================="
echo ""
echo "Estimated times:"
echo "  quick:      ~20 min (1 config)"
echo "  rank_focus: ~2 hours (6 configs)"
echo "  lr_focus:   ~2 hours (6 configs)"
echo "  default:    ~6 hours (18 configs)"
echo "  thorough:   ~40 hours (200 configs)"
echo ""
echo "Starting at: $(date)"
echo "Results will be saved after EACH config."
echo ""

# Run the Python sweep script
python3 python/train_sweep.py --preset "$PRESET"

echo ""
echo "Finished at: $(date)"
