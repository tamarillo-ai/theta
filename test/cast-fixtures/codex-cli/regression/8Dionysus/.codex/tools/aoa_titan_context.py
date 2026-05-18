#!/usr/bin/env python3
"""Emit Titan service-cohort context for Codex hooks.

This script does not spawn agents. It only returns additional context.
"""

import json

context = """Titan service cohort names are visible bearer names, not generic role shadows:
- Atlas: bearer_id=titan:atlas:founder, role_key=architect, read-only, active after explicit summon.
- Sentinel: bearer_id=titan:sentinel:founder, role_key=reviewer, read-only, active after explicit summon.
- Mneme: bearer_id=titan:mneme:founder, role_key=memory-keeper, read-only, active after explicit summon.
- Forge: bearer_id=titan:forge:founder, role_key=coder, workspace-write posture, locked until mutation payload gate.
- Delta: bearer_id=titan:delta:founder, role_key=evaluator, read-only, locked until judgment payload gate.

Do not autospawn subagents from hook context.
Do not spawn generic role agents as shadows for named Titan bearers.
Do not activate Forge without mutation payload: scope, expected_files, rollback_note, approval_ref, test_plan.
Do not activate Delta without judgment payload: claim, criteria, evidence_refs, verdict_scope.
Prefer creating or updating a local Titan session receipt through aoa-sdk/scripts/titanctl.py when the operator asks for runtime harness tracking.
"""

print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": context
    }
}))
