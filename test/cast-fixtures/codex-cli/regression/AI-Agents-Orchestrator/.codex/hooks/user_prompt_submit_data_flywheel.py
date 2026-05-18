#!/usr/bin/env python3
"""Codex UserPromptSubmit hook — logs prompts for analytics and adds project context."""

import json
import sys
from datetime import datetime, timezone


def main():
    data = json.load(sys.stdin)
    prompt = data.get("prompt", "")

    # Add project-specific context hints based on prompt content
    context_hints = []

    if any(word in prompt.lower() for word in ["test", "pytest", "spec"]):
        context_hints.append(
            "Tests are in tests/. Run: python -m pytest tests/ --override-ini='addopts=' -q --timeout=30 -m 'not integration and not slow'"
        )

    if any(word in prompt.lower() for word in ["adapter", "agent", "claude", "codex", "gemini"]):
        context_hints.append(
            "Adapters are in orchestrator/adapters/ and agentic_team/adapters/ (independent copies). Both extend BaseAdapter."
        )

    if any(word in prompt.lower() for word in ["report", "dashboard", "metric"]):
        context_hints.append(
            "Reports: orchestrator/observability/report_generator.py. Output in reports/. HTML dashboard uses Chart.js."
        )

    if any(word in prompt.lower() for word in ["config", "yaml", "workflow"]):
        context_hints.append(
            "Config: orchestrator/config/agents.yaml — agents, workflows, settings, agentic_team roles."
        )

    if context_hints:
        output = {
            "hookSpecificOutput": {
                "hookEventName": "UserPromptSubmit",
                "additionalContext": " ".join(context_hints),
            }
        }
        print(json.dumps(output))
    else:
        print(json.dumps({}))


if __name__ == "__main__":
    main()
