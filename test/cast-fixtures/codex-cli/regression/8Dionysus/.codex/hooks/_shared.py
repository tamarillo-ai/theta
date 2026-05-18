from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]


WORKSPACE_MARKER = "AOA_WORKSPACE_ROOT"
REPORT_DIR = Path(".codex") / "generated" / "hooks"


def workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def parse_event() -> dict[str, Any]:
    raw = sys.stdin.read().strip()
    if not raw:
        return {}
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        return {"_raw": raw}
    return payload if isinstance(payload, dict) else {"payload": payload}


def utc_stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def ensure_report_dir(root: Path | None = None) -> Path:
    target_root = root or workspace_root()
    path = target_root / REPORT_DIR
    path.mkdir(parents=True, exist_ok=True)
    return path


def write_event_log(event_name: str, payload: dict[str, Any]) -> str:
    root = workspace_root()
    try:
        report_dir = ensure_report_dir(root)
        path = report_dir / f"{event_name}_{utc_stamp()}.json"
        path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        return str(path)
    except OSError as exc:
        return f"log-unavailable:{exc.__class__.__name__}"


def load_project_config(root: Path | None = None) -> tuple[dict[str, Any] | None, str | None]:
    config_path = (root or workspace_root()) / ".codex" / "config.toml"
    if not config_path.exists():
        return None, "missing"
    try:
        return tomllib.loads(config_path.read_text(encoding="utf-8")), None
    except Exception as exc:  # pragma: no cover - defensive
        return None, str(exc)


def seam_report(root: Path | None = None) -> dict[str, Any]:
    target_root = root or workspace_root()
    config_path = target_root / ".codex" / "config.toml"
    hooks_path = target_root / ".codex" / "hooks.json"
    marker_path = target_root / WORKSPACE_MARKER
    config, config_error = load_project_config(target_root)
    features = config.get("features", {}) if isinstance(config, dict) else {}
    markers = config.get("project_root_markers", []) if isinstance(config, dict) else []

    return {
        "workspace_root": str(target_root),
        "workspace_marker": {
            "path": str(marker_path),
            "present": marker_path.exists(),
        },
        "project_config": {
            "path": str(config_path),
            "present": config_path.exists(),
            "parse_error": config_error,
            "project_root_markers": markers if isinstance(markers, list) else [],
            "codex_hooks_enabled": bool(features.get("codex_hooks")) if isinstance(features, dict) else False,
        },
        "project_hooks_json": {
            "path": str(hooks_path),
            "present": hooks_path.exists(),
        },
    }


def continue_payload(
    event_name: str,
    *,
    system_message: str | None = None,
    additional_context: str | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {"continue": True}
    if system_message:
        payload["systemMessage"] = system_message
    if additional_context:
        payload["hookSpecificOutput"] = {
            "hookEventName": event_name,
            "additionalContext": additional_context,
        }
    return payload
