#!/usr/bin/env python3
"""Codex PreToolUse hook — blocks dangerous bash commands."""

import json
import sys

BLOCKED_PATTERNS = [
    "rm -rf /",
    "rm -rf ~",
    "rm -rf .",
    "dd if=",
    "mkfs.",
    "> /dev/sd",
    ":(){ :|:& };:",
]


def main():
    data = json.load(sys.stdin)
    command = data.get("tool_input", {}).get("command", "")

    for pattern in BLOCKED_PATTERNS:
        if pattern in command:
            output = {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": f"Blocked by policy: command matches dangerous pattern '{pattern}'",
                }
            }
            print(json.dumps(output))
            return

    # Allow everything else
    print(json.dumps({}))


if __name__ == "__main__":
    main()
