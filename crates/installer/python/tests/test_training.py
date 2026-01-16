#!/usr/bin/env python3
"""
Tests for the installer LLM training pipeline.

Run with: python -m pytest tests/ -v
Or: python tests/test_training.py
"""

import json
import sys
from pathlib import Path

import pytest

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from train_lora import (
    load_training_data,
    format_example_for_training,
    SYSTEM_PROMPT,
    SHELL_COMMAND_TOOL,
)


# Path to training data
TRAINING_DIR = Path(__file__).parent.parent / "training"
MODEL_PATH = Path(__file__).parent.parent.parent.parent.parent / "vendor" / "models" / "SmolLM3-3B"


class TestTrainingDataLoading:
    """Tests for training data loading and validation."""

    def test_training_directory_exists(self):
        """Training directory should exist."""
        assert TRAINING_DIR.exists(), f"Training directory not found: {TRAINING_DIR}"

    def test_training_files_exist(self):
        """At least one JSONL training file should exist."""
        jsonl_files = list(TRAINING_DIR.glob("*.jsonl"))
        assert len(jsonl_files) > 0, "No JSONL training files found"

    def test_load_training_data(self):
        """Should load training data without errors."""
        examples = load_training_data(TRAINING_DIR)
        assert len(examples) > 0, "No training examples loaded"

    def test_training_data_has_required_fields(self):
        """All training examples should have a 'query' field."""
        examples = load_training_data(TRAINING_DIR)

        for i, ex in enumerate(examples):
            assert "query" in ex, f"Example {i} missing 'query' field: {ex}"

    def test_training_data_has_response_or_command(self):
        """Each example should have either 'response' or 'command'."""
        examples = load_training_data(TRAINING_DIR)

        for i, ex in enumerate(examples):
            has_response = "response" in ex
            has_command = "command" in ex
            has_context = "context" in ex  # For clarification examples
            has_plan = "plan" in ex  # For planning examples

            assert has_response or has_command or has_context or has_plan, \
                f"Example {i} has neither response, command, context, nor plan: {ex}"

    def test_minimum_training_examples(self):
        """Should have at least 1000 training examples."""
        examples = load_training_data(TRAINING_DIR)
        assert len(examples) >= 1000, f"Expected at least 1000 examples, got {len(examples)}"

    def test_no_duplicate_queries(self):
        """Check for exact duplicate queries (warning, not error)."""
        examples = load_training_data(TRAINING_DIR)
        queries = [ex["query"].lower().strip() for ex in examples]
        unique_queries = set(queries)

        duplicate_count = len(queries) - len(unique_queries)
        if duplicate_count > 0:
            # This is a warning, not a failure - some duplicates may be intentional
            print(f"Warning: Found {duplicate_count} duplicate queries")

    def test_valid_json_in_all_files(self):
        """All JSONL files should contain valid JSON."""
        errors = []

        for jsonl_file in TRAINING_DIR.glob("*.jsonl"):
            with open(jsonl_file) as f:
                for line_num, line in enumerate(f, 1):
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        json.loads(line)
                    except json.JSONDecodeError as e:
                        errors.append(f"{jsonl_file.name}:{line_num}: {e}")

        assert not errors, f"Invalid JSON found:\n" + "\n".join(errors)


class TestExampleFormatting:
    """Tests for training example formatting."""

    @pytest.fixture
    def tokenizer(self):
        """Load tokenizer if model exists, otherwise skip."""
        if not MODEL_PATH.exists():
            pytest.skip(f"Model not found at {MODEL_PATH}")

        from transformers import AutoTokenizer
        tokenizer = AutoTokenizer.from_pretrained(MODEL_PATH)
        if tokenizer.pad_token is None:
            tokenizer.pad_token = tokenizer.eos_token
        return tokenizer

    def test_format_command_example(self, tokenizer):
        """Command examples should format with function call syntax."""
        example = {"query": "list disks", "command": "lsblk"}
        formatted = format_example_for_training(example, tokenizer)

        assert "text" in formatted
        text = formatted["text"]

        # Should contain the query
        assert "list disks" in text

        # Should contain function call markers
        assert "run_shell_command" in text
        assert "lsblk" in text

    def test_format_response_example(self, tokenizer):
        """Text response examples should format correctly."""
        example = {"query": "hello", "response": "Hi there!"}
        formatted = format_example_for_training(example, tokenizer)

        assert "text" in formatted
        text = formatted["text"]

        # Should contain the query and response
        assert "hello" in text
        assert "Hi there" in text

    def test_format_includes_system_prompt(self, tokenizer):
        """Formatted examples should include the system prompt."""
        example = {"query": "test", "response": "test response"}
        formatted = format_example_for_training(example, tokenizer)

        # System prompt content should be present
        assert "LevitateOS installation assistant" in formatted["text"]

    def test_format_includes_tool_declaration(self, tokenizer):
        """Formatted examples should include tool declaration."""
        example = {"query": "test", "command": "ls"}
        formatted = format_example_for_training(example, tokenizer)

        # Tool declaration should be present
        assert "run_shell_command" in formatted["text"]
        assert "Execute a shell command" in formatted["text"]


class TestSystemPromptAndTools:
    """Tests for system prompt and tool definitions."""

    def test_system_prompt_not_empty(self):
        """System prompt should not be empty."""
        assert SYSTEM_PROMPT
        assert len(SYSTEM_PROMPT) > 100

    def test_system_prompt_mentions_capabilities(self):
        """System prompt should mention key capabilities."""
        assert "disk" in SYSTEM_PROMPT.lower()
        assert "partition" in SYSTEM_PROMPT.lower()
        assert "bootloader" in SYSTEM_PROMPT.lower()

    def test_shell_command_tool_structure(self):
        """Shell command tool should have correct structure."""
        assert SHELL_COMMAND_TOOL["type"] == "function"
        assert "function" in SHELL_COMMAND_TOOL

        func = SHELL_COMMAND_TOOL["function"]
        assert func["name"] == "run_shell_command"
        assert "description" in func
        assert "parameters" in func

        params = func["parameters"]
        assert "command" in params["properties"]
        assert "command" in params["required"]


class TestDatasetCreation:
    """Tests for HuggingFace dataset creation."""

    @pytest.fixture
    def tokenizer(self):
        """Load tokenizer if model exists, otherwise skip."""
        if not MODEL_PATH.exists():
            pytest.skip(f"Model not found at {MODEL_PATH}")

        from transformers import AutoTokenizer
        tokenizer = AutoTokenizer.from_pretrained(MODEL_PATH)
        if tokenizer.pad_token is None:
            tokenizer.pad_token = tokenizer.eos_token
        return tokenizer

    def test_create_small_dataset(self, tokenizer):
        """Should be able to create a dataset from examples."""
        from datasets import Dataset

        examples = load_training_data(TRAINING_DIR)[:10]
        formatted = [format_example_for_training(ex, tokenizer) for ex in examples]

        dataset = Dataset.from_list(formatted)
        assert len(dataset) == 10
        assert "text" in dataset.column_names

    def test_tokenize_examples(self, tokenizer):
        """Should be able to tokenize formatted examples."""
        examples = load_training_data(TRAINING_DIR)[:5]

        for ex in examples:
            formatted = format_example_for_training(ex, tokenizer)
            tokens = tokenizer(
                formatted["text"],
                truncation=True,
                max_length=512,
                return_tensors=None
            )

            assert "input_ids" in tokens
            assert len(tokens["input_ids"]) > 0
            assert len(tokens["input_ids"]) <= 512


class TestLLMServer:
    """Tests for the LLM server module."""

    def test_server_imports(self):
        """Server module should import without errors."""
        import llm_server

        assert hasattr(llm_server, "LLMServer")
        assert hasattr(llm_server, "gather_system_facts")
        assert hasattr(llm_server, "format_system_context")
        assert hasattr(llm_server, "build_system_prompt")

    def test_gather_system_facts(self):
        """Should gather system facts without crashing."""
        from llm_server import gather_system_facts

        facts = gather_system_facts()

        assert isinstance(facts, dict)
        assert "uefi" in facts
        assert "hostname" in facts
        assert "timezone" in facts

    def test_format_system_context(self):
        """Should format system context correctly."""
        from llm_server import format_system_context

        mock_facts = {
            "uefi": True,
            "network": True,
            "hostname": "test-host",
            "timezone": "UTC",
            "disks": {
                "blockdevices": [
                    {"name": "sda", "size": "500G", "type": "disk", "model": "Test SSD"}
                ]
            },
            "users": ["testuser"],
        }

        context = format_system_context(mock_facts)

        assert "UEFI" in context
        assert "Connected" in context
        assert "test-host" in context
        assert "UTC" in context
        assert "/dev/sda" in context
        assert "500G" in context
        assert "testuser" in context

    def test_build_system_prompt(self):
        """Should build system prompt with context."""
        from llm_server import build_system_prompt

        context = "## Test Context\n- Test item"
        prompt = build_system_prompt(context)

        assert "LevitateOS installation assistant" in prompt
        assert "Test Context" in prompt
        assert "IMPORTANT:" in prompt
        assert "hallucinate" in prompt.lower()


class TestAugmentData:
    """Tests for the data augmentation script."""

    def test_augment_imports(self):
        """Augment script should import without errors."""
        import augment_data

        assert hasattr(augment_data, "DISK_QUERIES")
        assert hasattr(augment_data, "TIMEZONES")
        assert hasattr(augment_data, "add_typos")
        assert hasattr(augment_data, "lowercase_variation")

    def test_add_typos_function(self):
        """Typo function should work correctly."""
        from augment_data import add_typos

        # With prob=0, should never add typos
        text = "hello world"
        result = add_typos(text, prob=0)
        assert result == text

        # With prob=1, should always modify (unless it's too short)
        # Run multiple times to check it does something
        modified_count = 0
        for _ in range(10):
            result = add_typos("hello world test", prob=1.0)
            if result != "hello world test":
                modified_count += 1

        assert modified_count > 0, "add_typos with prob=1 should modify text"

    def test_lowercase_variation_function(self):
        """Lowercase variation should return valid cases."""
        from augment_data import lowercase_variation

        text = "Hello World"

        # Run multiple times to check all variations
        results = set()
        for _ in range(100):
            results.add(lowercase_variation(text))

        # Should produce at least lowercase and original
        assert "hello world" in results or text in results or "HELLO WORLD" in results

    def test_timezone_mapping(self):
        """Timezone mapping should have valid paths."""
        from augment_data import TIMEZONES

        for city, tz_path in TIMEZONES.items():
            assert tz_path.startswith(("America/", "Europe/", "Asia/", "Australia/", "UTC"))


def run_tests():
    """Run all tests and report results."""
    import subprocess
    result = subprocess.run(
        [sys.executable, "-m", "pytest", str(Path(__file__).parent), "-v", "--tb=short"],
        cwd=Path(__file__).parent.parent
    )
    return result.returncode


if __name__ == "__main__":
    # Allow running directly without pytest
    exit(run_tests())
