#!/usr/bin/env python3
"""
LLM Runner - Natural language to shell command translation for LevitateOS.

Translates natural language queries into shell commands using FunctionGemma.
"""

import argparse
import json as json_module
import os
import re
import subprocess
import sys
import time
from pathlib import Path

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

# Optional: LoRA adapter support
try:
    from peft import PeftModel
    PEFT_AVAILABLE = True
except ImportError:
    PEFT_AVAILABLE = False

# =============================================================================
# Exit Codes
# =============================================================================
EXIT_SUCCESS = 0
EXIT_EXTRACTION_FAILED = 1
EXIT_MODEL_NOT_FOUND = 2
EXIT_MODEL_LOAD_FAILED = 3
EXIT_INVALID_ARGS = 4
EXIT_EXECUTION_FAILED = 5


# =============================================================================
# Color Support
# =============================================================================
class Colors:
    """ANSI color codes for terminal output."""
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    RED = "\033[31m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    BLUE = "\033[34m"
    CYAN = "\033[36m"

    _enabled = True

    @classmethod
    def disable(cls):
        """Disable all colors."""
        cls._enabled = False

    @classmethod
    def c(cls, code: str, text: str) -> str:
        """Apply color code to text if colors are enabled."""
        if cls._enabled:
            return f"{code}{text}{cls.RESET}"
        return text

    @classmethod
    def bold(cls, text: str) -> str:
        return cls.c(cls.BOLD, text)

    @classmethod
    def dim(cls, text: str) -> str:
        return cls.c(cls.DIM, text)

    @classmethod
    def red(cls, text: str) -> str:
        return cls.c(cls.RED, text)

    @classmethod
    def green(cls, text: str) -> str:
        return cls.c(cls.GREEN, text)

    @classmethod
    def yellow(cls, text: str) -> str:
        return cls.c(cls.YELLOW, text)

    @classmethod
    def cyan(cls, text: str) -> str:
        return cls.c(cls.CYAN, text)

    @classmethod
    def error(cls, text: str) -> str:
        return cls.c(cls.BOLD + cls.RED, text)

    @classmethod
    def command(cls, text: str) -> str:
        return cls.c(cls.BOLD + cls.YELLOW, text)


# =============================================================================
# Tool Definition
# =============================================================================
# System prompt required by FunctionGemma to activate function calling mode
# See: https://ai.google.dev/gemma/docs/functiongemma/formatting-and-best-practices
# Includes few-shot examples to help the 270M model map natural language to commands
SYSTEM_PROMPT = """You are a model that can do function calling with the following functions.

Translate user requests to shell commands. Common mappings:
files/folder -> ls -la
disk/storage -> df -h
memory/ram -> free -h
processes -> ps aux
time/date -> date
network -> ss -tuln
find X files -> find . -name "*.X"
search text -> grep -r "text" .

Always call run_shell_command with a valid Linux command."""

SHELL_COMMAND_TOOL = {
    "type": "function",
    "function": {
        "name": "run_shell_command",
        "description": """Execute a shell command on a Linux system. Use this function to:
- List files and directories (ls, dir, find)
- Show file contents (cat, head, tail, less)
- Search for text in files (grep, ripgrep, ag)
- Check disk usage and storage (df, du, free)
- Manage files (cp, mv, rm, mkdir, touch, chmod)
- View system information (uname, whoami, pwd, date, uptime)
- Process management (ps, top, kill)
- Network operations (ping, curl, wget, ip, ss)
- Archive and compress (tar, gzip, zip, unzip)
This function can execute ANY valid shell command. When the user asks about files, directories, system info, processes, or any task that can be done via command line, use this function.""",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The exact shell command to execute. Must be a valid Linux/Unix command."
                }
            },
            "required": ["command"]
        }
    }
}


# =============================================================================
# Output Functions
# =============================================================================
class Output:
    """Handles output based on format and verbosity settings."""

    def __init__(self, format_type: str, quiet: bool, verbose: bool, debug: bool):
        self.format = format_type
        self.quiet = quiet
        self.verbose = verbose
        self.debug = debug
        self._json_data = {}

    def status(self, message: str):
        """Print status message (respects quiet mode)."""
        if self.format == "raw":
            return
        if not self.quiet:
            print(Colors.dim(message), file=sys.stderr)

    def info(self, message: str):
        """Print info message (always shown except raw mode)."""
        if self.format == "raw":
            return
        print(message, file=sys.stderr)

    def error(self, message: str, error_type: str = "error"):
        """Print error message."""
        if self.format == "json":
            self._json_data["success"] = False
            self._json_data["error"] = error_type
            self._json_data["message"] = message
        elif self.format == "raw":
            print(f"error: {message}", file=sys.stderr)
        else:
            print(Colors.error(f"Error: {message}"), file=sys.stderr)

    def debug_msg(self, message: str):
        """Print debug message (only in debug mode)."""
        if self.debug and self.format != "raw":
            print(Colors.dim(f"[debug] {message}"), file=sys.stderr)

    def verbose_msg(self, message: str):
        """Print verbose message (only in verbose mode)."""
        if self.verbose and self.format != "raw":
            print(Colors.dim(f"[info] {message}"), file=sys.stderr)

    def command(self, cmd: str):
        """Output the generated command."""
        if self.format == "json":
            self._json_data["command"] = cmd
        elif self.format == "raw":
            print(cmd)
        else:
            print(f"{Colors.dim('Command:')} {Colors.command(cmd)}")

    def execution_result(self, stdout: str, stderr: str, returncode: int):
        """Output execution results."""
        if self.format == "json":
            self._json_data["executed"] = True
            self._json_data["exit_code"] = returncode
            self._json_data["stdout"] = stdout
            self._json_data["stderr"] = stderr
        else:
            if stdout:
                print(stdout, end="")
            if stderr:
                print(Colors.yellow(stderr), file=sys.stderr, end="")

    def set_json_field(self, key: str, value):
        """Set a field for JSON output."""
        self._json_data[key] = value

    def finalize(self, success: bool = True):
        """Finalize and print JSON output if in JSON mode."""
        if self.format == "json":
            if "success" not in self._json_data:
                self._json_data["success"] = success
            print(json_module.dumps(self._json_data, indent=2))


# =============================================================================
# Command Extraction
# =============================================================================
def extract_function_call(output: str, debug: bool = False) -> tuple[str | None, str | None]:
    """
    Extract the shell command from FunctionGemma's function call output.

    Returns:
        tuple: (command, error_reason) - command is None if extraction failed
    """
    # Pattern 1: Full function call format
    pattern1 = r'call:run_shell_command\{command:<escape>(.*?)<escape>\}'
    match = re.search(pattern1, output, re.DOTALL)
    if match:
        return match.group(1).strip(), None

    # Pattern 2: Simpler escape format
    pattern2 = r'command:<escape>(.*?)<escape>'
    match = re.search(pattern2, output, re.DOTALL)
    if match:
        return match.group(1).strip(), None

    # Pattern 3: Look for start_function_call marker
    if '<start_function_call>' in output:
        return None, "Found function call marker but could not parse command"

    # Check if model refused or gave conversational response
    refusal_patterns = [
        r"I('m| am) sorry",
        r"I cannot",
        r"I can't",
        r"could you please",
        r"would you like",
    ]
    for pattern in refusal_patterns:
        if re.search(pattern, output, re.IGNORECASE):
            return None, "Model gave conversational response instead of function call"

    return None, "No function call pattern found in output"


# =============================================================================
# Main Logic
# =============================================================================
def validate_model_path(path: str) -> tuple[bool, str]:
    """Validate that model path exists and has required files."""
    model_path = Path(path)

    if not model_path.exists():
        return False, f"Model path does not exist: {path}"

    if not model_path.is_dir():
        return False, f"Model path is not a directory: {path}"

    # Check for required files
    required_files = ["config.json"]
    for req in required_files:
        if not (model_path / req).exists():
            return False, f"Missing required file: {req}"

    # Check for model weights
    has_weights = any([
        (model_path / "model.safetensors").exists(),
        (model_path / "pytorch_model.bin").exists(),
        any(model_path.glob("model-*.safetensors")),
    ])
    if not has_weights:
        return False, "No model weights found (expected .safetensors or .bin)"

    return True, ""


def main():
    parser = argparse.ArgumentParser(
        description="Translate natural language to shell commands using FunctionGemma",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s -p "list all files"
  %(prog)s -p "find python files" --format json
  %(prog)s -p "show disk usage" --execute
  %(prog)s -p "remove temp files" --execute --confirm
  %(prog)s -p "partition disk" --adapter adapters/install.lora

Exit Codes:
  0  Success
  1  Could not extract command from model output
  2  Model not found
  3  Model load failure
  4  Invalid arguments
  5  Command execution failed
"""
    )

    # Required arguments
    parser.add_argument(
        "--prompt", "-p",
        required=True,
        help="Natural language prompt to translate"
    )

    # Model options
    parser.add_argument(
        "--model", "-m",
        default="/usr/lib/levitate/models/functiongemma",
        help="Path to the FunctionGemma model directory"
    )
    parser.add_argument(
        "--adapter", "-a",
        default=None,
        help="Path to LoRA adapter directory (optional)"
    )
    parser.add_argument(
        "--max-tokens", "-n",
        type=int,
        default=128,
        help="Maximum tokens to generate (default: 128)"
    )

    # Output format
    parser.add_argument(
        "--format", "-f",
        choices=["plain", "json", "raw"],
        default="plain",
        help="Output format: plain (colored), json (machine-readable), raw (just command)"
    )

    # Verbosity
    parser.add_argument(
        "--quiet", "-q",
        action="store_true",
        help="Suppress status messages"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Show additional information (timing, tokens)"
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Show debug information including raw model output"
    )

    # Execution options
    parser.add_argument(
        "--execute", "-x",
        action="store_true",
        help="Execute the generated command"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show command but don't execute (use with --execute)"
    )
    parser.add_argument(
        "--confirm", "-c",
        action="store_true",
        help="Ask for confirmation before executing"
    )

    # Display options
    parser.add_argument(
        "--no-color",
        action="store_true",
        help="Disable colored output"
    )

    args = parser.parse_args()

    # Handle color settings
    if args.no_color or os.environ.get("NO_COLOR") or not sys.stderr.isatty():
        Colors.disable()

    # Initialize output handler
    out = Output(args.format, args.quiet, args.verbose, args.debug)
    out.set_json_field("query", args.prompt)

    # Validate model path
    valid, error_msg = validate_model_path(args.model)
    if not valid:
        out.error(error_msg, "model_not_found")
        out.finalize(False)
        sys.exit(EXIT_MODEL_NOT_FOUND)

    # Load model and tokenizer
    out.status(f"Loading model from {args.model}...")
    start_time = time.time()

    try:
        tokenizer = AutoTokenizer.from_pretrained(args.model)
        model = AutoModelForCausalLM.from_pretrained(
            args.model,
            torch_dtype=torch.float32,
            device_map="auto"
        )

        # Load LoRA adapter if specified
        if args.adapter:
            if not PEFT_AVAILABLE:
                out.error("LoRA adapter specified but 'peft' library not installed. Run: pip install peft", "adapter_error")
                out.finalize(False)
                sys.exit(EXIT_MODEL_LOAD_FAILED)

            if not Path(args.adapter).exists():
                out.error(f"Adapter not found: {args.adapter}", "adapter_not_found")
                out.finalize(False)
                sys.exit(EXIT_MODEL_NOT_FOUND)

            out.status(f"Loading adapter from {args.adapter}...")
            model = PeftModel.from_pretrained(model, args.adapter)
            # Merge adapter for faster inference
            model = model.merge_and_unload()
            out.verbose_msg("Adapter loaded and merged")

    except Exception as e:
        out.error(f"Failed to load model: {e}", "model_load_failed")
        out.finalize(False)
        sys.exit(EXIT_MODEL_LOAD_FAILED)

    load_time = time.time() - start_time
    out.verbose_msg(f"Model loaded in {load_time:.2f}s")

    # Build messages with system prompt (required for FunctionGemma)
    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": args.prompt}
    ]

    # Apply chat template with tools
    try:
        inputs = tokenizer.apply_chat_template(
            messages,
            tools=[SHELL_COMMAND_TOOL],
            add_generation_prompt=True,
            return_dict=True,
            return_tensors="pt"
        )
    except Exception as e:
        out.error(f"Failed to apply chat template: {e}", "template_error")
        out.finalize(False)
        sys.exit(EXIT_MODEL_LOAD_FAILED)

    # Move to model device
    inputs = {k: v.to(model.device) for k, v in inputs.items()}

    # Generate
    out.status("Generating...")
    gen_start = time.time()

    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=args.max_tokens,
            do_sample=True,
            temperature=0.7,
            top_k=40,
            top_p=0.95,
            pad_token_id=tokenizer.eos_token_id
        )

    gen_time = time.time() - gen_start

    # Decode output
    generated_ids = outputs[0][inputs["input_ids"].shape[1]:]
    output_text = tokenizer.decode(generated_ids, skip_special_tokens=False)
    tokens_generated = len(generated_ids)

    out.verbose_msg(f"Generated {tokens_generated} tokens in {gen_time:.2f}s")
    out.debug_msg(f"Raw output: {output_text}")

    out.set_json_field("tokens_generated", tokens_generated)
    out.set_json_field("generation_time_ms", int(gen_time * 1000))

    # Extract command
    command, error_reason = extract_function_call(output_text, args.debug)

    if not command:
        out.error(f"Could not extract command: {error_reason}", "extraction_failed")
        out.set_json_field("raw_output", output_text)
        out.finalize(False)
        sys.exit(EXIT_EXTRACTION_FAILED)

    # Output the command
    out.command(command)
    out.set_json_field("executed", False)

    # Handle execution
    if args.execute:
        if args.dry_run:
            out.info(Colors.dim("(dry-run mode - command not executed)"))
            out.set_json_field("dry_run", True)
        else:
            # Confirmation prompt
            if args.confirm:
                if args.format == "raw":
                    out.info("Confirmation required but not available in raw mode")
                    out.finalize(True)
                    sys.exit(EXIT_SUCCESS)

                try:
                    response = input(f"{Colors.yellow('Execute?')} [y/N] ")
                    if response.lower() != 'y':
                        out.info("Cancelled.")
                        out.set_json_field("cancelled", True)
                        out.finalize(True)
                        sys.exit(EXIT_SUCCESS)
                except (EOFError, KeyboardInterrupt):
                    out.info("\nCancelled.")
                    out.finalize(True)
                    sys.exit(EXIT_SUCCESS)

            # Execute the command
            out.status(f"Executing: {command}")
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True
            )

            out.execution_result(result.stdout, result.stderr, result.returncode)

            if result.returncode != 0:
                out.finalize(False)
                sys.exit(EXIT_EXECUTION_FAILED)

    out.finalize(True)
    sys.exit(EXIT_SUCCESS)


if __name__ == "__main__":
    main()
