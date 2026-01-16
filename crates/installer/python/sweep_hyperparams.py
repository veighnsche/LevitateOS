#!/usr/bin/env python3
"""
Hyperparameter sweep for LoRA training.

Trains multiple configurations and evaluates each on the test set.
Results are saved to a JSON file for comparison.

Usage:
    python sweep_hyperparams.py --model vendor/models/SmolLM3-3B --quick
"""

import argparse
import json
import subprocess
import sys
import shutil
from pathlib import Path
from datetime import datetime
from itertools import product

# Script directory for resolving relative paths
SCRIPT_DIR = Path(__file__).parent.resolve()

# Hyperparameter search space
SEARCH_SPACE = {
    "learning_rate": [1e-4, 2e-4, 5e-4],
    "epochs": [1, 2, 3],
    "lora_r": [8, 16, 32],
    "lora_alpha": [16, 32],  # Will also try 2x rank
}

# Quick search (fewer combinations for testing)
QUICK_SEARCH = {
    "learning_rate": [2e-4],
    "epochs": [1, 2],
    "lora_r": [8, 16],
    "lora_alpha": [16, 32],
}


def run_training(model_path: str, output_dir: str, data_dir: str, config: dict, use_4bit: bool = False) -> dict:
    """Run training with given config, return training stats."""
    train_script = SCRIPT_DIR / "train_lora.py"
    cmd = [
        sys.executable, str(train_script),
        "--model", model_path,
        "--output", output_dir,
        "--data-dir", data_dir,
        "--epochs", str(config["epochs"]),
        "--learning-rate", str(config["learning_rate"]),
        "--lora-r", str(config["lora_r"]),
        "--lora-alpha", str(config["lora_alpha"]),
    ]

    if use_4bit:
        cmd.append("--use-4bit")

    print(f"\n{'='*60}")
    print(f"Training config: {config}")
    print(f"Command: {' '.join(cmd)}")
    print("="*60)

    result = subprocess.run(cmd, capture_output=True, text=True, cwd=SCRIPT_DIR)

    if result.returncode != 0:
        print(f"Training failed!")
        print(result.stderr)
        return {"success": False, "error": result.stderr}

    # Parse training output for final loss
    train_loss = None
    eval_loss = None
    for line in result.stdout.split('\n'):
        if "'loss'" in line:
            try:
                # Extract loss from trainer output
                import re
                match = re.search(r"'loss':\s*([\d.]+)", line)
                if match:
                    train_loss = float(match.group(1))
            except:
                pass
        if "'eval_loss'" in line:
            try:
                import re
                match = re.search(r"'eval_loss':\s*([\d.]+)", line)
                if match:
                    eval_loss = float(match.group(1))
            except:
                pass

    return {
        "success": True,
        "train_loss": train_loss,
        "eval_loss": eval_loss,
    }


def run_evaluation(model_path: str, adapter_path: str, test_file: str, max_examples: int = 100, use_4bit: bool = False) -> dict:
    """Run evaluation on test set, return metrics."""
    test_script = SCRIPT_DIR / "test_model.py"
    cmd = [
        sys.executable, str(test_script),
        "--model", model_path,
        "--adapter", adapter_path,
        "--test-file", test_file,
        "--max-examples", str(max_examples),
    ]

    if use_4bit:
        cmd.append("--use-4bit")

    print(f"\nEvaluating {adapter_path}...")

    result = subprocess.run(cmd, capture_output=True, text=True, cwd=SCRIPT_DIR)

    if result.returncode != 0:
        print(f"Evaluation failed!")
        print(result.stderr)
        return {"success": False, "error": result.stderr}

    # Parse evaluation output
    metrics = {"success": True}

    for line in result.stdout.split('\n'):
        if "Response Type Accuracy:" in line:
            try:
                metrics["type_accuracy"] = float(line.split(':')[1].strip().rstrip('%'))
            except:
                pass
        if "Exact match:" in line:
            try:
                # "Exact match: 45/50 (90.0%)"
                import re
                match = re.search(r'\(([\d.]+)%\)', line)
                if match:
                    metrics["command_exact_match"] = float(match.group(1))
            except:
                pass

    return metrics


def generate_configs(search_space: dict) -> list:
    """Generate all hyperparameter combinations."""
    keys = list(search_space.keys())
    values = list(search_space.values())

    configs = []
    for combo in product(*values):
        config = dict(zip(keys, combo))
        configs.append(config)

    # Also add configs where alpha = 2 * rank
    extra_configs = []
    for config in configs:
        if config["lora_alpha"] != config["lora_r"] * 2:
            new_config = config.copy()
            new_config["lora_alpha"] = config["lora_r"] * 2
            if new_config not in configs and new_config not in extra_configs:
                extra_configs.append(new_config)

    return configs + extra_configs


def main():
    parser = argparse.ArgumentParser(description="Hyperparameter sweep for LoRA")
    parser.add_argument("--model", "-m", default="vendor/models/SmolLM3-3B",
                        help="Base model path")
    parser.add_argument("--output-dir", "-o", default=None,
                        help="Directory to save sweep results (default: sweep_results/)")
    parser.add_argument("--quick", action="store_true",
                        help="Quick sweep with fewer combinations")
    parser.add_argument("--test-examples", type=int, default=100,
                        help="Number of test examples to evaluate")
    parser.add_argument("--use-4bit", action="store_true",
                        help="Use 4-bit quantization (saves memory but lower quality)")
    args = parser.parse_args()

    # Resolve all paths to absolute
    # Model path: check if relative to cwd or script dir
    model_path = Path(args.model)
    if not model_path.is_absolute():
        # Try relative to cwd first, then relative to script dir's parent (project root)
        if not model_path.exists():
            model_path = SCRIPT_DIR.parent.parent.parent / args.model
        model_path = model_path.resolve()

    if not model_path.exists():
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    # Output dir
    if args.output_dir:
        output_dir = Path(args.output_dir)
        if not output_dir.is_absolute():
            output_dir = SCRIPT_DIR / output_dir
    else:
        output_dir = SCRIPT_DIR / "sweep_results"
    output_dir = output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    # Training data dir
    data_dir = (SCRIPT_DIR / "training").resolve()
    if not data_dir.exists():
        print(f"Error: Training data not found at {data_dir}", file=sys.stderr)
        print("Run augment_data.py first to generate training data.", file=sys.stderr)
        sys.exit(1)

    # Test file
    test_file = (SCRIPT_DIR / "testing" / "test_dataset.jsonl").resolve()
    if not test_file.exists():
        print(f"Error: Test data not found at {test_file}", file=sys.stderr)
        print("Run augment_data.py first to generate test data.", file=sys.stderr)
        sys.exit(1)

    print(f"Model: {model_path}")
    print(f"Training data: {data_dir}")
    print(f"Test data: {test_file}")
    print(f"Output: {output_dir}")
    print(f"Quantization: {'4-bit' if args.use_4bit else 'fp16 (full precision)'}")

    # Generate configs
    search_space = QUICK_SEARCH if args.quick else SEARCH_SPACE
    configs = generate_configs(search_space)

    print(f"Hyperparameter sweep: {len(configs)} configurations")
    print(f"Search space: {search_space}")

    results = []
    best_result = None
    best_score = -1

    for i, config in enumerate(configs):
        print(f"\n{'#'*60}")
        print(f"# Configuration {i+1}/{len(configs)}")
        print(f"{'#'*60}")

        # Create unique output dir for this config
        config_name = f"lr{config['learning_rate']}_e{config['epochs']}_r{config['lora_r']}_a{config['lora_alpha']}"
        adapter_dir = output_dir / config_name

        # Clean previous run
        if adapter_dir.exists():
            shutil.rmtree(adapter_dir)

        # Train
        train_result = run_training(
            str(model_path),
            str(adapter_dir),
            str(data_dir),
            config,
            use_4bit=args.use_4bit
        )

        if not train_result["success"]:
            results.append({
                "config": config,
                "train": train_result,
                "eval": None,
                "score": 0,
            })
            continue

        # Evaluate
        eval_result = run_evaluation(
            str(model_path),
            str(adapter_dir),
            str(test_file),
            max_examples=args.test_examples,
            use_4bit=args.use_4bit
        )

        # Calculate combined score
        # Prioritize command accuracy, then type accuracy
        score = 0
        if eval_result.get("success"):
            score = (
                eval_result.get("command_exact_match", 0) * 0.7 +
                eval_result.get("type_accuracy", 0) * 0.3
            )

        result = {
            "config": config,
            "train": train_result,
            "eval": eval_result,
            "score": score,
        }
        results.append(result)

        if score > best_score:
            best_score = score
            best_result = result

        print(f"\nConfig score: {score:.1f}")
        print(f"Current best: {best_score:.1f}")

    # Save results
    results_file = output_dir / f"sweep_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    with open(results_file, "w") as f:
        json.dump({
            "search_space": search_space,
            "results": results,
            "best": best_result,
        }, f, indent=2)

    # Print summary
    print("\n" + "="*60)
    print("SWEEP COMPLETE")
    print("="*60)

    print(f"\nResults saved to: {results_file}")

    print("\n--- Top 5 Configurations ---")
    sorted_results = sorted(results, key=lambda x: x["score"], reverse=True)
    for i, r in enumerate(sorted_results[:5]):
        print(f"\n{i+1}. Score: {r['score']:.1f}")
        print(f"   Config: lr={r['config']['learning_rate']}, epochs={r['config']['epochs']}, r={r['config']['lora_r']}, alpha={r['config']['lora_alpha']}")
        if r["eval"] and r["eval"].get("success"):
            print(f"   Type acc: {r['eval'].get('type_accuracy', 'N/A')}%, Cmd exact: {r['eval'].get('command_exact_match', 'N/A')}%")
        if r["train"]:
            print(f"   Eval loss: {r['train'].get('eval_loss', 'N/A')}")

    if best_result:
        print("\n" + "="*60)
        print("BEST CONFIGURATION")
        print("="*60)
        print(f"Config: {best_result['config']}")
        print(f"Score: {best_result['score']:.1f}")
        print(f"\nTo use this config:")
        print(f"  python train_lora.py \\")
        print(f"    --learning-rate {best_result['config']['learning_rate']} \\")
        print(f"    --epochs {best_result['config']['epochs']} \\")
        print(f"    --lora-r {best_result['config']['lora_r']} \\")
        print(f"    --lora-alpha {best_result['config']['lora_alpha']}")


if __name__ == "__main__":
    main()
