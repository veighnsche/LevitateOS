#!/usr/bin/env python3
"""
LLM Server - HTTP server for LevitateOS installer.

Loads the model once, serves requests via HTTP.
Gathers and injects real system facts to prevent hallucination.
"""

import argparse
import json
import os
import re
import subprocess
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

# Optional: LoRA support
try:
    from peft import PeftModel
    PEFT_AVAILABLE = True
except ImportError:
    PEFT_AVAILABLE = False


def gather_system_facts() -> dict:
    """Gather real system state - disks, mounts, users, etc."""
    facts = {}

    # Disks
    try:
        result = subprocess.run(
            ["lsblk", "-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE,MODEL"],
            capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            facts["disks"] = json.loads(result.stdout)
    except Exception as e:
        facts["disks_error"] = str(e)

    # Boot mode (UEFI vs BIOS)
    facts["uefi"] = os.path.exists("/sys/firmware/efi/efivars")

    # Current mounts (only if /mnt exists and has something mounted)
    try:
        if os.path.ismount("/mnt"):
            result = subprocess.run(
                ["findmnt", "-J", "-R", "/mnt"],
                capture_output=True, text=True, timeout=5
            )
            if result.returncode == 0 and result.stdout.strip():
                facts["mounts"] = json.loads(result.stdout)
            else:
                facts["mounts"] = None
        else:
            facts["mounts"] = None
    except Exception:
        facts["mounts"] = None

    # Network connectivity
    try:
        result = subprocess.run(
            ["ping", "-c", "1", "-W", "2", "archlinux.org"],
            capture_output=True, timeout=5
        )
        facts["network"] = result.returncode == 0
    except Exception:
        facts["network"] = False

    # Current hostname
    try:
        facts["hostname"] = subprocess.run(
            ["hostname"], capture_output=True, text=True, timeout=2
        ).stdout.strip()
    except Exception:
        facts["hostname"] = "unknown"

    # Timezone
    try:
        tz_link = os.readlink("/etc/localtime")
        facts["timezone"] = tz_link.replace("/usr/share/zoneinfo/", "")
    except Exception:
        facts["timezone"] = "not set"

    # Existing users (non-system)
    try:
        users = []
        with open("/etc/passwd") as f:
            for line in f:
                parts = line.strip().split(":")
                if len(parts) >= 7:
                    uid = int(parts[2])
                    if 1000 <= uid < 60000:
                        users.append(parts[0])
        facts["users"] = users
    except Exception:
        facts["users"] = []

    return facts


def format_system_context(facts: dict) -> str:
    """Format system facts into a context string for the LLM."""
    lines = ["## Current System State\n"]

    # Boot mode
    if facts.get("uefi"):
        lines.append("- Boot mode: UEFI")
    else:
        lines.append("- Boot mode: Legacy BIOS")

    # Network
    if facts.get("network"):
        lines.append("- Network: Connected")
    else:
        lines.append("- Network: Not connected")

    # Hostname
    lines.append(f"- Hostname: {facts.get('hostname', 'unknown')}")

    # Timezone
    lines.append(f"- Timezone: {facts.get('timezone', 'not set')}")

    # Disks
    if "disks" in facts and "blockdevices" in facts["disks"]:
        lines.append("\n## Available Disks\n")
        for dev in facts["disks"]["blockdevices"]:
            if dev.get("type") == "disk":
                model = (dev.get("model") or "").strip() or "Unknown"
                lines.append(f"- /dev/{dev['name']}: {dev['size']} ({model})")
                if "children" in dev:
                    for part in dev["children"]:
                        mp = part.get("mountpoint", "")
                        fs = part.get("fstype", "")
                        mount_info = f" mounted at {mp}" if mp else ""
                        fs_info = f" [{fs}]" if fs else ""
                        lines.append(f"  - /dev/{part['name']}: {part['size']}{fs_info}{mount_info}")

    # Current mounts
    if facts.get("mounts"):
        lines.append("\n## Current Mounts\n")
        lines.append("Target partitions are mounted under /mnt")

    # Users
    if facts.get("users"):
        lines.append(f"\n## Existing Users: {', '.join(facts['users'])}")

    return "\n".join(lines)


def build_system_prompt(system_context: str) -> str:
    """Build the full system prompt with injected facts.

    Note: We don't use /no_think prefix so the model shows its reasoning.
    The thinking content is extracted and shown to users for transparency.
    """
    return f"""You are the LevitateOS installation assistant. Help users install their operating system.

{system_context}

CRITICAL RULES:
1. When user wants to DO something (list, format, partition, mount, create, set, install), ALWAYS call run_shell_command
2. When user CONFIRMS an action (yes, ok, proceed, continue, do it), EXECUTE the pending command via run_shell_command
3. When user asks a QUESTION (what is, how do, should I, explain), respond with text

COMMAND REFERENCE:
- List disks: lsblk
- Partition disk: sgdisk -Z /dev/X && sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 /dev/X
- Format EFI: mkfs.fat -F32 /dev/X1
- Format root: mkfs.ext4 /dev/X2
- Mount root: mount /dev/X2 /mnt
- Mount EFI: mkdir -p /mnt/boot/efi && mount /dev/X1 /mnt/boot/efi
- Set hostname: hostnamectl set-hostname NAME
- Set timezone: timedatectl set-timezone ZONE
- Create user: useradd -m -G wheel NAME
- Install GRUB: grub-install --target=x86_64-efi --efi-directory=/boot/efi

Only reference disks that exist in the system state above. Never hallucinate disk names."""

SHELL_COMMAND_TOOL = {
    "type": "function",
    "function": {
        "name": "run_shell_command",
        "description": "Execute a shell command for system installation tasks.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "The shell command to execute"}
            },
            "required": ["command"]
        }
    }
}


class LLMServer:
    def __init__(self, model_path: str, adapter_path: str = None):
        print(f"Loading model from {model_path}...", file=sys.stderr)
        self.tokenizer = AutoTokenizer.from_pretrained(model_path)
        self.model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=torch.float32,
            device_map="auto"
        )

        # Load LoRA adapter if specified
        if adapter_path and os.path.exists(adapter_path):
            if not PEFT_AVAILABLE:
                print("Warning: peft not installed, cannot load LoRA adapter", file=sys.stderr)
            else:
                print(f"Loading LoRA adapter from {adapter_path}...", file=sys.stderr)
                self.model = PeftModel.from_pretrained(self.model, adapter_path)
                print("LoRA adapter loaded.", file=sys.stderr)

        # Get the device where the model is loaded
        self.device = next(self.model.parameters()).device
        print(f"Model loaded on {self.device}.", file=sys.stderr)

        # Cache system facts (refresh on each query)
        self._cached_facts = None
        self._valid_disks = set()  # For verification

    def _refresh_system_facts(self) -> str:
        """Gather fresh system facts and cache valid disk names."""
        facts = gather_system_facts()
        self._cached_facts = facts

        # Build set of valid disk/partition names for verification
        self._valid_disks = set()
        if "disks" in facts and "blockdevices" in facts["disks"]:
            for dev in facts["disks"]["blockdevices"]:
                if dev.get("type") == "disk":
                    self._valid_disks.add(f"/dev/{dev['name']}")
                    if "children" in dev:
                        for part in dev["children"]:
                            self._valid_disks.add(f"/dev/{part['name']}")

        return format_system_context(facts)

    def generate(self, messages: list[dict], max_tokens: int = 256) -> dict:
        """
        Generate response for a conversation.

        Args:
            messages: List of {"role": "user"|"assistant", "content": "..."} dicts
                      representing the conversation history.
            max_tokens: Maximum tokens to generate.
        """
        # Refresh system facts and inject into system prompt
        system_context = self._refresh_system_facts()
        system_prompt = build_system_prompt(system_context)

        # Build full message list with system prompt first
        full_messages = [{"role": "system", "content": system_prompt}]
        full_messages.extend(messages)

        inputs = self.tokenizer.apply_chat_template(
            full_messages,
            tools=[SHELL_COMMAND_TOOL],
            add_generation_prompt=True,
            return_dict=True,
            return_tensors="pt"
        )
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        with torch.no_grad():
            outputs = self.model.generate(
                **inputs,
                max_new_tokens=max_tokens,
                do_sample=True,
                temperature=0.3,  # Low temperature for deterministic, factual outputs
                top_p=0.9,
                pad_token_id=self.tokenizer.eos_token_id
            )

        generated_ids = outputs[0][inputs["input_ids"].shape[1]:]
        raw_output = self.tokenizer.decode(generated_ids, skip_special_tokens=False)

        result = self._extract_response(raw_output)

        # Post-generation verification: check for hallucinated disks
        result = self._verify_response(result)

        return result

    def _extract_response(self, output: str) -> dict:
        # Extract thinking content if present
        thinking = None
        think_match = re.search(r'<think>(.*?)</think>', output, re.DOTALL)
        if think_match:
            thinking = think_match.group(1).strip()
            if not thinking:  # Empty thinking block
                thinking = None

        # Check for SmolLM3 XML-style tool call
        tool_call_match = re.search(r'<tool_call>\s*(\{[^}]+\})\s*</tool_call>', output, re.DOTALL)
        if tool_call_match:
            try:
                tool_data = json.loads(tool_call_match.group(1))
                if tool_data.get("name") == "run_shell_command":
                    cmd = tool_data.get("arguments", {}).get("command", "")
                    result = {"success": True, "type": "command", "command": cmd.strip()}
                    if thinking:
                        result["thinking"] = thinking
                    return result
            except json.JSONDecodeError:
                pass

        # Natural language response - strip XML tags but preserve content
        text = re.sub(r'<think>.*?</think>', '', output, flags=re.DOTALL)  # Remove think block
        text = re.sub(r'<[^>]+>', '', text).strip()  # Remove other tags
        result = {"success": True, "type": "text", "response": text}
        if thinking:
            result["thinking"] = thinking
        return result

    def _verify_response(self, result: dict) -> dict:
        """Verify that response doesn't reference non-existent disks/partitions."""
        if result.get("type") == "command":
            command = result.get("command", "")

            # Extract any /dev/* paths from the command
            dev_paths = re.findall(r'/dev/\w+', command)

            for path in dev_paths:
                # Allow common pseudo-devices
                if path in ["/dev/null", "/dev/zero", "/dev/urandom", "/dev/random"]:
                    continue

                if path not in self._valid_disks:
                    # Hallucinated disk detected!
                    return {
                        "success": True,
                        "type": "text",
                        "response": f"I couldn't find {path} on this system. Let me check what disks are available.",
                        "warning": f"Blocked hallucinated disk: {path}",
                        "suggested_command": "lsblk"
                    }

        return result


# Global server instance
llm_server = None


class RequestHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/query":
            self.send_error(404)
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length).decode("utf-8")

        try:
            data = json.loads(body)
            messages = data.get("messages", [])
            max_tokens = data.get("max_tokens", 256)

            result = llm_server.generate(messages, max_tokens)

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(result).encode("utf-8"))

        except Exception as e:
            self.send_response(500)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"success": False, "error": str(e)}).encode("utf-8"))

    def log_message(self, format, *args):
        print(f"[request] {args[0]}", file=sys.stderr)


def main():
    global llm_server

    parser = argparse.ArgumentParser(description="LLM HTTP Server")
    parser.add_argument("--model", "-m", default="vendor/models/SmolLM3-3B", help="Model path")
    parser.add_argument("--adapter", "-a", default=None, help="LoRA adapter path (optional)")
    parser.add_argument("--port", "-p", type=int, default=8765, help="Port to listen on")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    args = parser.parse_args()

    model_path = Path(args.model)
    if not model_path.exists():
        print(f"Error: Model not found at {args.model}", file=sys.stderr)
        sys.exit(1)

    llm_server = LLMServer(args.model, adapter_path=args.adapter)

    server = HTTPServer((args.host, args.port), RequestHandler)
    print(f"Server listening on http://{args.host}:{args.port}", file=sys.stderr)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.", file=sys.stderr)
        server.shutdown()


if __name__ == "__main__":
    main()
