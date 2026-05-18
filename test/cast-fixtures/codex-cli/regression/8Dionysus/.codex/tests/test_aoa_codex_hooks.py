from __future__ import annotations

import json
import sys
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1] / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from aoa_codex_hooks.events import (  # noqa: E402
    handle_session_start,
    handle_stop,
    handle_user_prompt_submit,
)
from aoa_codex_hooks.state import (  # noqa: E402
    build_hook_report,
    prompt_mentions_codex_surfaces,
)


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def make_workspace(tmp_path: Path) -> Path:
    write_text(tmp_path / "AOA_WORKSPACE_ROOT", "")
    write_text(
        tmp_path / ".codex" / "config.toml",
        'project_root_markers = ["AOA_WORKSPACE_ROOT", ".git"]\n\n[features]\ncodex_hooks = true\n',
    )
    write_text(
        tmp_path / ".codex" / "hooks.json",
        json.dumps(
            {
                "hooks": {
                    "SessionStart": [
                        {
                            "matcher": "startup|resume",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": f"python3 {tmp_path}/.codex/hooks/aoa_session_start.py",
                                }
                            ],
                        }
                    ]
                }
            }
        ),
    )
    for name in ["aoa_session_start.py", "aoa_user_prompt_submit.py", "aoa_stop_doctor.py"]:
        write_text(tmp_path / ".codex" / "hooks" / name, "#!/usr/bin/env python3\n")
    return tmp_path


def test_build_hook_report_ready(tmp_path: Path) -> None:
    workspace = make_workspace(tmp_path)
    report = build_hook_report(workspace)
    assert report["ready"] is True
    assert report["error_count"] == 0


def test_build_hook_report_detects_placeholder(tmp_path: Path) -> None:
    workspace = make_workspace(tmp_path)
    write_text(
        workspace / ".codex" / "hooks.json",
        json.dumps(
            {
                "hooks": {
                    "Stop": [
                        {
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "python3 __AOA_WORKSPACE_ROOT__/.codex/hooks/aoa_stop_doctor.py",
                                }
                            ]
                        }
                    ]
                }
            }
        ),
    )
    report = build_hook_report(workspace)
    assert report["ready"] is True
    hooks_surface = next(surface for surface in report["surfaces"] if surface["name"] == "project_hooks_json")
    assert hooks_surface["status"] == "warn"


def test_prompt_detection() -> None:
    assert prompt_mentions_codex_surfaces("Wire MCP and subagents for Codex") is True
    assert prompt_mentions_codex_surfaces("just refactor this parser") is False


def test_session_start_returns_context(tmp_path: Path) -> None:
    workspace = make_workspace(tmp_path)
    payload = handle_session_start({"source": "startup"}, workspace)
    assert payload["continue"] is True
    assert payload["hookSpecificOutput"]["hookEventName"] == "SessionStart"


def test_session_start_includes_titan_incarnation_context_when_present(tmp_path: Path) -> None:
    workspace = make_workspace(tmp_path)
    write_text(
        workspace / ".codex" / "tools" / "aoa_titan_incarnation_context.py",
        "#!/usr/bin/env python3\n",
    )
    payload = handle_session_start({"source": "startup"}, workspace)
    context = payload["hookSpecificOutput"]["additionalContext"]
    assert "Titan Incarnation Spine source is present" in context
    assert "No autospawn" in context


def test_user_prompt_submit_on_match(tmp_path: Path) -> None:
    workspace = make_workspace(tmp_path)
    payload = handle_user_prompt_submit({"prompt": "Please wire MCP for Codex"}, workspace)
    assert payload["hookSpecificOutput"]["hookEventName"] == "UserPromptSubmit"


def test_stop_writes_reports(tmp_path: Path) -> None:
    workspace = make_workspace(tmp_path)
    payload = handle_stop({"stop_hook_active": False}, workspace)
    assert payload["continue"] is True
    report_dir = workspace / ".codex" / "generated" / "codex" / "hooks"
    assert any(report_dir.glob("aoa_codex_hooks_report_*.json"))
    assert (report_dir / "aoa_codex_hooks_report_latest.json").exists() is False
    assert any(report_dir.glob("aoa_codex_hooks_report_*_latest.json"))
