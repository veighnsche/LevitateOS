#!/usr/bin/env python3
"""
LLM Inference Server - HTTP server for local model inference.

Loads a HuggingFace model once, serves requests via HTTP.
Supports LoRA adapters and tool/function calling.

Usage:
    python llm_server.py --model path/to/model --port 8765
    python llm_server.py --model path/to/model --adapter path/to/lora

API:
    POST /generate
    {
        "messages": [{"role": "user", "content": "Hello"}],
        "max_tokens": 256,
        "temperature": 0.7,
        "tools": [...]  // optional
    }

    POST /health
    Returns {"status": "ok", "model": "..."}
"""

import argparse
import json
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from typing import Optional

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

# Optional: LoRA support
try:
    from peft import PeftModel
    PEFT_AVAILABLE = True
except ImportError:
    PEFT_AVAILABLE = False


class LLMServer:
    def __init__(
        self,
        model_path: str,
        adapter_path: Optional[str] = None,
        device: Optional[str] = None,
        dtype: str = "auto"
    ):
        print(f"Loading model from {model_path}...", file=sys.stderr)

        # Determine dtype
        if dtype == "auto":
            torch_dtype = torch.float16 if torch.cuda.is_available() else torch.float32
        elif dtype == "float16":
            torch_dtype = torch.float16
        elif dtype == "bfloat16":
            torch_dtype = torch.bfloat16
        else:
            torch_dtype = torch.float32

        self.tokenizer = AutoTokenizer.from_pretrained(model_path)
        self.model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=torch_dtype,
            device_map=device or "auto"
        )

        # Load LoRA adapter if specified
        if adapter_path and Path(adapter_path).exists():
            if not PEFT_AVAILABLE:
                print("Warning: peft not installed, cannot load LoRA adapter", file=sys.stderr)
            else:
                print(f"Loading LoRA adapter from {adapter_path}...", file=sys.stderr)
                self.model = PeftModel.from_pretrained(self.model, adapter_path)
                print("LoRA adapter loaded.", file=sys.stderr)

        self.device = next(self.model.parameters()).device
        self.model_path = model_path
        print(f"Model loaded on {self.device}.", file=sys.stderr)

        # Ensure padding token
        if self.tokenizer.pad_token is None:
            self.tokenizer.pad_token = self.tokenizer.eos_token
            self.tokenizer.pad_token_id = self.tokenizer.eos_token_id

    def generate(
        self,
        messages: list[dict],
        max_tokens: int = 256,
        temperature: float = 0.7,
        top_p: float = 0.9,
        tools: Optional[list] = None,
        system_prompt: Optional[str] = None
    ) -> dict:
        """Generate response for a conversation."""
        try:
            # Build message list
            full_messages = []
            if system_prompt:
                full_messages.append({"role": "system", "content": system_prompt})
            full_messages.extend(messages)

            # Apply chat template
            template_kwargs = {
                "add_generation_prompt": True,
                "return_dict": True,
                "return_tensors": "pt"
            }
            if tools:
                template_kwargs["tools"] = tools

            inputs = self.tokenizer.apply_chat_template(
                full_messages,
                **template_kwargs
            )
            inputs = {k: v.to(self.device) for k, v in inputs.items()}

            # Generate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=max_tokens,
                    do_sample=temperature > 0,
                    temperature=temperature if temperature > 0 else None,
                    top_p=top_p,
                    pad_token_id=self.tokenizer.eos_token_id
                )

            # Decode only the generated tokens
            generated_ids = outputs[0][inputs["input_ids"].shape[1]:]
            response = self.tokenizer.decode(generated_ids, skip_special_tokens=True)

            return {"success": True, "response": response.strip()}

        except Exception as e:
            return {"success": False, "error": str(e)}


# Global server instance
llm_server: Optional[LLMServer] = None


class RequestHandler(BaseHTTPRequestHandler):
    def _send_json(self, data: dict, status: int = 200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode("utf-8"))

    def do_POST(self):
        if self.path == "/health":
            self._send_json({
                "status": "ok",
                "model": llm_server.model_path if llm_server else None
            })
            return

        if self.path != "/generate":
            self._send_json({"error": "Not found"}, 404)
            return

        try:
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode("utf-8")
            data = json.loads(body)

            messages = data.get("messages", [])
            max_tokens = data.get("max_tokens", 256)
            temperature = data.get("temperature", 0.7)
            top_p = data.get("top_p", 0.9)
            tools = data.get("tools")
            system_prompt = data.get("system_prompt")

            result = llm_server.generate(
                messages=messages,
                max_tokens=max_tokens,
                temperature=temperature,
                top_p=top_p,
                tools=tools,
                system_prompt=system_prompt
            )

            self._send_json(result)

        except json.JSONDecodeError as e:
            self._send_json({"success": False, "error": f"Invalid JSON: {e}"}, 400)
        except Exception as e:
            self._send_json({"success": False, "error": str(e)}, 500)

    def do_GET(self):
        if self.path == "/health":
            self._send_json({
                "status": "ok",
                "model": llm_server.model_path if llm_server else None
            })
        else:
            self._send_json({"error": "Use POST /generate"}, 405)

    def log_message(self, format, *args):
        print(f"[{self.address_string()}] {args[0]}", file=sys.stderr)


def main():
    global llm_server

    parser = argparse.ArgumentParser(description="LLM HTTP Inference Server")
    parser.add_argument("--model", "-m", required=True, help="Model path or HuggingFace ID")
    parser.add_argument("--adapter", "-a", default=None, help="LoRA adapter path (optional)")
    parser.add_argument("--port", "-p", type=int, default=8765, help="Port to listen on")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--device", "-d", default=None, help="Device (cuda, cpu, auto)")
    parser.add_argument("--dtype", default="auto", choices=["auto", "float16", "bfloat16", "float32"],
                        help="Model dtype")
    args = parser.parse_args()

    model_path = Path(args.model)
    if not model_path.exists() and "/" not in args.model:
        print(f"Error: Model not found at {args.model}", file=sys.stderr)
        sys.exit(1)

    llm_server = LLMServer(
        args.model,
        adapter_path=args.adapter,
        device=args.device,
        dtype=args.dtype
    )

    server = HTTPServer((args.host, args.port), RequestHandler)
    print(f"Server listening on http://{args.host}:{args.port}", file=sys.stderr)
    print(f"  POST /generate - Generate text", file=sys.stderr)
    print(f"  GET  /health   - Health check", file=sys.stderr)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.", file=sys.stderr)
        server.shutdown()


if __name__ == "__main__":
    main()
