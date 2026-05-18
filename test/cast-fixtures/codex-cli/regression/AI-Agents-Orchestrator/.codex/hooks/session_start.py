#!/usr/bin/env python3
"""Codex SessionStart hook — loads project context at session launch."""

import json
import sys
from pathlib import Path


def main():
    data = json.load(sys.stdin)
    cwd = data.get("cwd", ".")

    context_parts = [
        "AI Coding Tools Orchestrator — dual system project.",
        "orchestrator/ = step-based workflows (implement -> review -> refine).",
        "agentic_team/ = free-communication team runtime (PM, Architect, Dev, QA, DevOps).",
        "ZERO shared code between them. Never import across.",
    ]

    # Check if agents.yaml exists and summarize
    config_path = Path(cwd) / "orchestrator" / "config" / "agents.yaml"
    if config_path.exists():
        context_parts.append(f"Config: {config_path}")

    # Check agent availability
    import shutil

    for tool in ["claude", "codex", "gemini"]:
        if shutil.which(tool):
            context_parts.append(f"Agent available: {tool}")

    output = {
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": " ".join(context_parts),
        }
    }
    print(json.dumps(output))


if __name__ == "__main__":
    main()
