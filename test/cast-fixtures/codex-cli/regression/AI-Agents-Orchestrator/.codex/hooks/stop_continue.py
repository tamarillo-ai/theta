#!/usr/bin/env python3
"""Codex Stop hook — checks if tests should be run before stopping."""

import json
import sys


def main():
    data = json.load(sys.stdin)
    last_message = data.get("last_assistant_message", "") or ""
    stop_hook_active = data.get("stop_hook_active", False)

    # Don't loop — only trigger once
    if stop_hook_active:
        print(json.dumps({}))
        return

    # If the assistant modified Python files, suggest running tests
    if any(
        phrase in last_message.lower()
        for phrase in ["created", "modified", "updated", "fixed", "implemented", "refactored"]
    ):
        if any(ext in last_message for ext in [".py", "test_", "adapter", "engine"]):
            output = {
                "decision": "block",
                "reason": "Run the test suite to verify the changes: python -m pytest tests/ --override-ini='addopts=' -q --timeout=30 -m 'not integration and not slow'",
            }
            print(json.dumps(output))
            return

    print(json.dumps({}))


if __name__ == "__main__":
    main()
