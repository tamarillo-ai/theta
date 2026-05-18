#!/usr/bin/env python3
"""Codex PostToolUse hook — reviews bash output for common issues."""

import json
import sys

WARNING_PATTERNS = {
    "SyntaxError": "Python syntax error detected in output.",
    "ModuleNotFoundError": "Missing Python module. May need pip install.",
    "PermissionError": "Permission denied. Check file permissions.",
    "FAILED": "Test failures detected in output.",
    "Error:": "Error message found in command output.",
}


def main():
    data = json.load(sys.stdin)
    tool_response = str(data.get("tool_response", ""))

    warnings = []
    for pattern, message in WARNING_PATTERNS.items():
        if pattern in tool_response:
            warnings.append(message)

    if warnings:
        output = {
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": "Hook warnings: " + " | ".join(warnings),
            }
        }
        print(json.dumps(output))
    else:
        print(json.dumps({}))


if __name__ == "__main__":
    main()
