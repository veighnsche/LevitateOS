#!/usr/bin/env python3
"""
Hyperparameter sweep for LoRA training.

Trains multiple LoRA adapters with different configurations to find the best one.

Usage:
    python train_sweep.py                    # Run default sweep
    python train_sweep.py --quick            # Quick sweep (fewer configs)
    python train_sweep.py --thorough         # Thorough sweep (more configs)
"""

import argparse
import itertools
import json
import queue
import re
import subprocess
import sys
import threading
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


@dataclass
class SweepConfig:
    """Configuration for a single training run."""
    rank: int
    alpha: int
    learning_rate: float
    epochs: int = 3
    batch_size: int = 4

    @property
    def name(self) -> str:
        lr_str = f"{self.learning_rate:.0e}".replace("-", "")
        return f"r{self.rank}_a{self.alpha}_lr{lr_str}_e{self.epochs}"


# Predefined sweep configurations
SWEEP_PRESETS = {
    "quick": {
        "ranks": [16],
        "alphas": [32],
        "learning_rates": [2e-4],
        "epochs": [2],
    },
    "default": {
        "ranks": [8, 16, 32],
        "alphas": [16, 32, 64],
        "learning_rates": [1e-4, 2e-4],
        "epochs": [3],
    },
    "thorough": {
        "ranks": [4, 8, 16, 32, 64],
        "alphas": [8, 16, 32, 64, 128],
        "learning_rates": [5e-5, 1e-4, 2e-4, 5e-4],
        "epochs": [3, 5],
    },
    "rank_focus": {
        "ranks": [4, 8, 16, 32, 64, 128],
        "alphas": [32],
        "learning_rates": [2e-4],
        "epochs": [3],
    },
    "lr_focus": {
        "ranks": [16],
        "alphas": [32],
        "learning_rates": [1e-5, 5e-5, 1e-4, 2e-4, 5e-4, 1e-3],
        "epochs": [3],
    },
}


def generate_configs(preset: str) -> list[SweepConfig]:
    """Generate all configurations for a sweep preset."""
    params = SWEEP_PRESETS[preset]

    configs = []
    for rank, alpha, lr, epochs in itertools.product(
        params["ranks"],
        params["alphas"],
        params["learning_rates"],
        params["epochs"],
    ):
        configs.append(SweepConfig(
            rank=rank,
            alpha=alpha,
            learning_rate=lr,
            epochs=epochs,
        ))

    return configs


def run_training(config: SweepConfig, model_path: Path, output_dir: Path, data_dir: Path, use_4bit: bool = True) -> dict:
    """Run a single training configuration."""

    cmd = [
        sys.executable, str(Path(__file__).parent / "train_lora.py"),
        "--model", str(model_path),
        "--output", str(output_dir),
        "--data-dir", str(data_dir),
        "--epochs", str(config.epochs),
        "--batch-size", str(config.batch_size),
        "--lora-r", str(config.rank),
        "--lora-alpha", str(config.alpha),
        "--learning-rate", str(config.learning_rate),
    ]

    if use_4bit:
        cmd.append("--use-4bit")

    log_file = output_dir.parent / f"{config.name}.log"

    result = {
        "config": config.name,
        "rank": config.rank,
        "alpha": config.alpha,
        "learning_rate": config.learning_rate,
        "epochs": config.epochs,
        "success": False,
        "eval_loss": None,
        "log_file": str(log_file),
    }

    def stream_output(process, log_file, done_event):
        """Stream process output to console and log file."""
        try:
            with open(log_file, "w") as log:
                for line in process.stdout:
                    print(f"    {line}", end="")
                    log.write(line)
                    log.flush()
        finally:
            done_event.set()

    try:
        log_file.parent.mkdir(parents=True, exist_ok=True)

        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )

        # Stream output in a separate thread so we can timeout
        done_event = threading.Event()
        reader_thread = threading.Thread(
            target=stream_output,
            args=(process, log_file, done_event),
            daemon=True
        )
        reader_thread.start()

        # Wait for completion with timeout (2 hours)
        if done_event.wait(timeout=7200):
            process.wait()
            result["success"] = process.returncode == 0
        else:
            # Timeout - kill the process
            process.kill()
            process.wait()
            result["error"] = "Timeout (2h)"
            return result

        # Parse eval loss from log
        if result["success"] and log_file.exists():
            log_content = log_file.read_text()
            matches = re.findall(r"'eval_loss': ([0-9.]+)", log_content)
            if matches:
                result["eval_loss"] = float(matches[-1])

    except Exception as e:
        result["error"] = str(e)

    return result


def run_evaluation(adapter_path: Path, model_path: Path) -> dict:
    """Evaluate a single adapter and return metrics."""
    try:
        cmd = [
            sys.executable, str(Path(__file__).parent / "evaluate_lora.py"),
            "--model", str(model_path),
            "--adapter", str(adapter_path),
            "--output", str(adapter_path / "eval_results.json"),
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)

        if result.returncode == 0 and (adapter_path / "eval_results.json").exists():
            with open(adapter_path / "eval_results.json") as f:
                eval_data = json.load(f)
                if eval_data:
                    return eval_data[0]
    except Exception as e:
        return {"error": str(e)}

    return {"error": "Evaluation failed"}


def main():
    parser = argparse.ArgumentParser(description="LoRA hyperparameter sweep")
    parser.add_argument("--preset", choices=list(SWEEP_PRESETS.keys()), default="default",
                        help="Sweep preset to use")
    parser.add_argument("--quick", action="store_const", const="quick", dest="preset",
                        help="Quick sweep (1 config)")
    parser.add_argument("--thorough", action="store_const", const="thorough", dest="preset",
                        help="Thorough sweep (many configs)")
    parser.add_argument("--model", "-m", default="../../../vendor/models/FunctionGemma",
                        help="Base model path")
    parser.add_argument("--data-dir", "-d", default="training",
                        help="Training data directory")
    parser.add_argument("--output-base", "-o", default="adapters",
                        help="Base output directory for adapters")
    parser.add_argument("--no-4bit", action="store_true",
                        help="Disable 4-bit quantization (uses more memory)")
    args = parser.parse_args()

    script_dir = Path(__file__).parent
    model_path = (script_dir / args.model).resolve()
    data_dir = (script_dir / args.data_dir).resolve()
    output_base = (script_dir / args.output_base).resolve()

    if not model_path.exists():
        print(f"Error: Model not found at {model_path}", file=sys.stderr)
        sys.exit(1)

    if not data_dir.exists():
        print(f"Error: Training data not found at {data_dir}", file=sys.stderr)
        sys.exit(1)

    # Generate configurations
    configs = generate_configs(args.preset)

    print("=" * 60)
    print(f"  LoRA Hyperparameter Sweep")
    print(f"  Preset: {args.preset}")
    print(f"  Total configurations: {len(configs)}")
    print("=" * 60)
    print()

    # Create output directory
    output_base.mkdir(parents=True, exist_ok=True)

    # Results tracking
    results = []
    results_file = output_base / "sweep_results.json"

    for i, config in enumerate(configs, 1):
        print(f"\n{'=' * 60}")
        print(f"[{i}/{len(configs)}] Training: {config.name}")
        print(f"  rank={config.rank}, alpha={config.alpha}, lr={config.learning_rate}, epochs={config.epochs}")
        print("=" * 60)

        output_dir = output_base / config.name

        # Train
        result = run_training(config, model_path, output_dir, data_dir, use_4bit=not args.no_4bit)

        if result["success"]:
            print(f"  ✓ Training complete! eval_loss={result['eval_loss']}")

            # Evaluate immediately after training
            print(f"  Evaluating on test queries...")
            eval_result = run_evaluation(output_dir, model_path)

            if "error" not in eval_result:
                result["command_accuracy"] = eval_result.get("command_accuracy", 0)
                result["score"] = eval_result.get("score", 0)
                print(f"  ✓ Evaluation complete! command_accuracy={100 * result['command_accuracy']:.1f}%")
            else:
                print(f"  ✗ Evaluation failed: {eval_result.get('error')}")
                result["eval_error"] = eval_result.get("error")
        else:
            print(f"  ✗ Training failed: {result.get('error', 'Unknown error')}")

        results.append(result)

        # Save results after EACH config (so we don't lose progress on crash)
        with open(results_file, "w") as f:
            json.dump(results, f, indent=2)

        # Print running leaderboard
        successful = [r for r in results if r.get("command_accuracy") is not None]
        if successful:
            successful.sort(key=lambda x: x.get("command_accuracy", 0), reverse=True)
            print(f"\n  Current best: {successful[0]['config']} ({100 * successful[0]['command_accuracy']:.1f}% accuracy)")

    # Final summary
    print("\n" + "=" * 60)
    print("  SWEEP COMPLETE!")
    print("=" * 60)

    successful = [r for r in results if r.get("success")]
    evaluated = [r for r in results if r.get("command_accuracy") is not None]
    failed = [r for r in results if not r.get("success")]

    print(f"\nTrained: {len(successful)}/{len(results)}")
    print(f"Evaluated: {len(evaluated)}/{len(results)}")

    if evaluated:
        evaluated.sort(key=lambda x: x.get("command_accuracy", 0), reverse=True)

        print("\n🏆 TOP 5 CONFIGURATIONS (by command accuracy):")
        for i, r in enumerate(evaluated[:5], 1):
            acc = r.get("command_accuracy", 0)
            loss = r.get("eval_loss", float("inf"))
            print(f"  {i}. {r['config']}: {100 * acc:.1f}% accuracy (eval_loss={loss:.4f})")

        best = evaluated[0]
        print(f"\n{'=' * 60}")
        print(f"🏆 BEST: {best['config']}")
        print(f"   Command accuracy: {100 * best['command_accuracy']:.1f}%")
        print(f"   Training eval_loss: {best.get('eval_loss', 'N/A')}")
        print(f"   Adapter path: {output_base / best['config']}")
        print("=" * 60)

    if failed:
        print(f"\nFailed configurations: {len(failed)}")
        for r in failed:
            print(f"  - {r['config']}: {r.get('error', 'Unknown')}")

    print(f"\nResults saved to: {results_file}")


if __name__ == "__main__":
    main()
