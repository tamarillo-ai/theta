from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .state import (
    build_hook_report,
    ensure_report_dir,
    prompt_mentions_codex_surfaces,
    report_to_markdown,
    write_report_files,
)


def parse_event(stdin_text: str) -> dict[str, Any]:
    stdin_text = stdin_text.strip()
    if not stdin_text:
        return {}
    return json.loads(stdin_text)


def utc_stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def _event_log_path(workspace_root: Path, event_name: str) -> Path:
    report_dir = ensure_report_dir(workspace_root)
    return report_dir / f"{event_name}_{utc_stamp()}.json"


def write_event_log(workspace_root: Path, event_name: str, payload: dict[str, Any]) -> str:
    log_path = _event_log_path(workspace_root, event_name)
    log_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return str(log_path)


def _trim_message(message: str, *, max_chars: int = 240) -> str:
    if len(message) <= max_chars:
        return message
    return message[: max_chars - 1].rstrip() + "…"


def _titan_incarnation_context(workspace_root: Path) -> str:
    context_tool = workspace_root / ".codex" / "tools" / "aoa_titan_incarnation_context.py"
    if not context_tool.exists():
        return ""
    return (
        " Titan Incarnation Spine source is present: summon named custom agents "
        "Atlas/Sentinel/Mneme, not generic architect/reviewer/memory-keeper "
        "shadows. Forge requires a mutation payload gate; Delta requires a "
        "judgment payload gate. No autospawn."
    )


def handle_session_start(event: dict[str, Any], workspace_root: Path) -> dict[str, Any]:
    report = build_hook_report(workspace_root)
    log_path = write_event_log(workspace_root, "session_start", {"event": event, "report": report})

    missing = [surface["name"] for surface in report["surfaces"] if surface["status"] == "error"]
    if missing:
        system_message = f"AoA hook seam is partial: {', '.join(missing)}."
        additional_context = (
            "AoA workspace was detected, but the hook/control-plane seam is incomplete. "
            "Keep edits workspace-root anchored and run the Codex doctor before changing .codex, .agents, skills, MCP, or subagent wiring. "
            f"Latest hook log: {log_path}."
        )
    else:
        system_message = "AoA hook seam is active."
        additional_context = (
            "AoA workspace hook seam is active. For repo law use AGENTS.md, for reusable workflows use skills, "
            "for live derived context use MCP, and for role delegation use subagents. "
            f"Latest hook log: {log_path}."
        )

    additional_context += _titan_incarnation_context(workspace_root)

    return {
        "continue": True,
        "systemMessage": system_message,
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": additional_context,
        },
    }


KEYWORD_GUIDANCE = {
    "mcp": "Use MCP for live or derived context instead of flattening it into prompts.",
    "skill": "Use skills for reusable bounded workflows and keep adapter/export seams aligned.",
    "skills": "Use skills for reusable bounded workflows and keep adapter/export seams aligned.",
    "plugin": "Use plugins as installable distribution units, not as new owners of meaning.",
    "plugins": "Use plugins as installable distribution units, not as new owners of meaning.",
    "subagent": "Use subagents for role-bearing delegation; keep MCP and skills inherited from the parent session when possible.",
    "subagents": "Use subagents for role-bearing delegation; keep MCP and skills inherited from the parent session when possible.",
    "agents.md": "Use AGENTS.md for workspace or repo law and conventions.",
    ".codex": "Keep .codex changes workspace-root anchored and validate after edits.",
    ".agents": "Keep .agents surfaces synchronized with exported skills and plugin market files.",
    "hook": "Hooks are experimental and best kept deterministic, narrow, and fail-open.",
    "hooks": "Hooks are experimental and best kept deterministic, narrow, and fail-open.",
}


def handle_user_prompt_submit(event: dict[str, Any], workspace_root: Path) -> dict[str, Any]:
    prompt = str(event.get("prompt") or "")
    payload = {"event": event}
    log_path = write_event_log(workspace_root, "user_prompt_submit", payload)

    if not prompt_mentions_codex_surfaces(prompt):
        return {"continue": True}

    lowered = prompt.lower()
    matched = [hint for token, hint in KEYWORD_GUIDANCE.items() if token in lowered]
    if not matched:
        matched = [
            "Keep Codex-control-plane edits workspace-root anchored.",
            "Run the doctor after structural changes to hooks, MCP, plugins, or subagents.",
        ]

    additional_context = (
        "AoA Codex crosswalk reminder: "
        + " ".join(dict.fromkeys(matched))
        + f" Latest hook log: {log_path}."
    )
    return {
        "continue": True,
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": _trim_message(additional_context, max_chars=600),
        },
    }


def handle_stop(
    event: dict[str, Any],
    workspace_root: Path,
    *,
    autocontinue_critical: bool = False,
) -> dict[str, Any]:
    report = build_hook_report(workspace_root)
    stamp = utc_stamp()
    paths = write_report_files(workspace_root, report, stem=f"aoa_codex_hooks_report_{stamp}")
    event_log = {
        "event": event,
        "report": report,
        "report_paths": paths,
        "markdown_preview": report_to_markdown(report),
    }
    log_path = write_event_log(workspace_root, "stop", event_log)

    if autocontinue_critical and report["error_count"] > 0 and not bool(event.get("stop_hook_active")):
        return {
            "decision": "block",
            "reason": (
                "Run one more pass over the AoA Codex seam: inspect the latest hooks report, fix the critical errors, "
                "and summarize what changed before stopping."
            ),
        }

    system_message = None
    if report["error_count"] > 0:
        system_message = (
            f"AoA hook doctor found {report['error_count']} critical seam issue(s). "
            f"See {paths['latest_markdown']} and {log_path}."
        )
    elif report["warn_count"] > 0:
        system_message = (
            f"AoA hook doctor found {report['warn_count']} warning(s). "
            f"See {paths['latest_markdown']} and {log_path}."
        )

    payload: dict[str, Any] = {"continue": True}
    if system_message:
        payload["systemMessage"] = _trim_message(system_message, max_chars=500)
    return payload
