from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]


WORKSPACE_MARKER = "AOA_WORKSPACE_ROOT"
DEFAULT_REPORT_DIR = Path(".codex") / "generated" / "codex" / "hooks"
HOOK_SCRIPT_NAMES = [
    "aoa_session_start.py",
    "aoa_user_prompt_submit.py",
    "aoa_stop_doctor.py",
]
TRIGGER_WORDS = {
    "codex",
    "mcp",
    "skill",
    "skills",
    "plugin",
    "plugins",
    "subagent",
    "subagents",
    "agent",
    "agents.md",
    ".codex",
    ".agents",
    "workspace",
    "hook",
    "hooks",
}


@dataclass(frozen=True)
class Surface:
    name: str
    status: str
    summary: str
    path: str | None = None
    details: dict[str, Any] | None = None

    def as_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "name": self.name,
            "status": self.status,
            "summary": self.summary,
        }
        if self.path is not None:
            payload["path"] = self.path
        if self.details:
            payload["details"] = self.details
        return payload


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def load_toml(path: Path) -> dict[str, Any]:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def find_workspace_root(start: Path) -> Path:
    current = start.resolve()
    for candidate in [current, *current.parents]:
        if (candidate / WORKSPACE_MARKER).exists():
            return candidate
    return current


def ensure_report_dir(workspace_root: Path) -> Path:
    report_dir = workspace_root / DEFAULT_REPORT_DIR
    report_dir.mkdir(parents=True, exist_ok=True)
    return report_dir


def user_hooks_path() -> Path:
    return Path.home() / ".codex" / "hooks.json"


def _contains_project_root_marker(config: dict[str, Any]) -> bool:
    markers = config.get("project_root_markers")
    if not isinstance(markers, list):
        return False
    return WORKSPACE_MARKER in {str(item) for item in markers}


def _safe_read_config(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    if not path.exists():
        return None, None
    try:
        return load_toml(path), None
    except Exception as exc:  # pragma: no cover - defensive
        return None, str(exc)


def _hook_paths(workspace_root: Path) -> list[Path]:
    hooks_dir = workspace_root / ".codex" / "hooks"
    return [hooks_dir / name for name in HOOK_SCRIPT_NAMES]


def _parse_hook_commands(hooks_path: Path) -> list[str]:
    if not hooks_path.exists():
        return []
    try:
        payload = load_json(hooks_path)
    except Exception:
        return []
    hooks = payload.get("hooks", {})
    commands: list[str] = []
    if not isinstance(hooks, dict):
        return commands
    for groups in hooks.values():
        if not isinstance(groups, list):
            continue
        for group in groups:
            if not isinstance(group, dict):
                continue
            handlers = group.get("hooks", [])
            if not isinstance(handlers, list):
                continue
            for handler in handlers:
                if isinstance(handler, dict) and handler.get("type") == "command":
                    command = handler.get("command")
                    if isinstance(command, str):
                        commands.append(command)
    return commands


def build_hook_report(workspace_root: Path) -> dict[str, Any]:
    workspace_root = workspace_root.resolve()
    config_path = workspace_root / ".codex" / "config.toml"
    hooks_path = workspace_root / ".codex" / "hooks.json"
    convergence_path = workspace_root / ".codex" / "generated" / "codex" / "aoa_codex_convergence_report.json"
    report_dir = workspace_root / DEFAULT_REPORT_DIR

    config, config_error = _safe_read_config(config_path)
    features = (config or {}).get("features", {}) if config else {}
    codex_hooks_enabled = isinstance(features, dict) and bool(features.get("codex_hooks"))
    project_root_marker_present = _contains_project_root_marker(config or {}) if config else False

    surfaces: list[Surface] = []

    marker_path = workspace_root / WORKSPACE_MARKER
    surfaces.append(
        Surface(
            name="workspace_marker",
            status="ok" if marker_path.exists() else "error",
            summary="Workspace marker anchors sibling-root discovery." if marker_path.exists() else "Missing AOA_WORKSPACE_ROOT marker.",
            path=str(marker_path),
        )
    )

    if config_path.exists() and config_error is None:
        surfaces.append(
            Surface(
                name="project_config",
                status="ok",
                summary="Project-scoped Codex config is present.",
                path=str(config_path),
            )
        )
    elif config_error:
        surfaces.append(
            Surface(
                name="project_config",
                status="error",
                summary=f"Project config exists but could not be parsed: {config_error}",
                path=str(config_path),
            )
        )
    else:
        surfaces.append(
            Surface(
                name="project_config",
                status="error",
                summary="Missing project-scoped .codex/config.toml.",
                path=str(config_path),
            )
        )

    surfaces.append(
        Surface(
            name="hooks_feature_flag",
            status="ok" if codex_hooks_enabled else "error",
            summary="features.codex_hooks is enabled." if codex_hooks_enabled else "features.codex_hooks is missing or false.",
            path=str(config_path),
        )
    )

    surfaces.append(
        Surface(
            name="project_root_markers",
            status="ok" if project_root_marker_present else "warn",
            summary="project_root_markers contains AOA_WORKSPACE_ROOT."
            if project_root_marker_present
            else "project_root_markers does not mention AOA_WORKSPACE_ROOT.",
            path=str(config_path),
        )
    )

    commands = _parse_hook_commands(hooks_path)
    has_placeholder = any("__AOA_WORKSPACE_ROOT__" in command for command in commands)
    if not hooks_path.exists():
        surfaces.append(
            Surface(
                name="project_hooks_json",
                status="error",
                summary="Missing project .codex/hooks.json.",
                path=str(hooks_path),
            )
        )
    elif not commands:
        surfaces.append(
            Surface(
                name="project_hooks_json",
                status="warn",
                summary="hooks.json exists but no command hooks were parsed.",
                path=str(hooks_path),
            )
        )
    else:
        status = "warn" if has_placeholder else "ok"
        summary = "hooks.json exists with rendered commands." if not has_placeholder else "hooks.json still contains placeholder workspace paths."
        surfaces.append(
            Surface(
                name="project_hooks_json",
                status=status,
                summary=summary,
                path=str(hooks_path),
                details={"commands": commands},
            )
        )

    hook_paths = _hook_paths(workspace_root)
    missing_scripts = [str(path) for path in hook_paths if not path.exists()]
    surfaces.append(
        Surface(
            name="hook_scripts",
            status="ok" if not missing_scripts else "error",
            summary="All AoA hook scripts are present." if not missing_scripts else f"Missing hook scripts: {', '.join(missing_scripts)}",
            path=str(workspace_root / ".codex" / "hooks"),
        )
    )

    uhooks = user_hooks_path()
    surfaces.append(
        Surface(
            name="user_hooks_overlap",
            status="warn" if uhooks.exists() else "ok",
            summary="User-level hooks.json also exists; both layers will run." if uhooks.exists() else "No user-level hooks.json detected.",
            path=str(uhooks),
        )
    )

    report_dir_exists = report_dir.exists()
    surfaces.append(
        Surface(
            name="generated_hooks_dir",
            status="ok" if report_dir_exists else "warn",
            summary=".codex/generated/codex/hooks already exists." if report_dir_exists else ".codex/generated/codex/hooks will be created on first hook run.",
            path=str(report_dir),
        )
    )

    if convergence_path.exists():
        try:
            convergence = load_json(convergence_path)
            convergence_ready = bool(convergence.get("ready"))
            surfaces.append(
                Surface(
                    name="convergence_report",
                    status="ok" if convergence_ready else "warn",
                    summary="Latest convergence report is ready." if convergence_ready else "Latest convergence report is not ready.",
                    path=str(convergence_path),
                )
            )
        except Exception as exc:  # pragma: no cover - defensive
            surfaces.append(
                Surface(
                    name="convergence_report",
                    status="warn",
                    summary=f"Convergence report exists but could not be parsed: {exc}",
                    path=str(convergence_path),
                )
            )
    else:
        surfaces.append(
            Surface(
                name="convergence_report",
                status="warn",
                summary="No convergence report found yet.",
                path=str(convergence_path),
            )
        )

    recommendations: list[str] = []
    if not marker_path.exists():
        recommendations.append("Create AOA_WORKSPACE_ROOT at the intended sibling-workspace root.")
    if not config_path.exists():
        recommendations.append("Add .codex/config.toml at the workspace root and trust the project.")
    if config_path.exists() and not codex_hooks_enabled:
        recommendations.append("Enable [features].codex_hooks = true in project config.")
    if config_path.exists() and not project_root_marker_present:
        recommendations.append("Add AOA_WORKSPACE_ROOT to project_root_markers so root discovery reaches the workspace root.")
    if hooks_path.exists() and has_placeholder:
        recommendations.append("Render hooks.json with absolute workspace-root paths instead of the placeholder token.")
    if not hooks_path.exists():
        recommendations.append("Install project-level .codex/hooks.json next to the active config layer.")
    if missing_scripts:
        recommendations.append("Install the AoA hook scripts under .codex/hooks/ and the helper module under .codex/tools/aoa_codex_hooks/.")
    if uhooks.exists():
        recommendations.append("Audit user-level hooks.json to avoid duplicate hook execution across layers.")

    error_count = sum(1 for surface in surfaces if surface.status == "error")
    warn_count = sum(1 for surface in surfaces if surface.status == "warn")

    ready = error_count == 0
    return {
        "workspace_root": str(workspace_root),
        "ready": ready,
        "error_count": error_count,
        "warn_count": warn_count,
        "surfaces": [surface.as_dict() for surface in surfaces],
        "recommendations": recommendations,
        "environment": {
            "platform": os.name,
            "user_hooks_path": str(uhooks),
        },
    }


def report_to_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# AoA Codex hooks report",
        "",
        f"- workspace_root: `{report['workspace_root']}`",
        f"- ready: `{report['ready']}`",
        f"- errors: `{report['error_count']}`",
        f"- warnings: `{report['warn_count']}`",
        "",
        "## Surfaces",
        "",
    ]
    for surface in report["surfaces"]:
        path = surface.get("path")
        path_suffix = f" (`{path}`)" if path else ""
        lines.append(f"- **{surface['name']}** [{surface['status']}] {surface['summary']}{path_suffix}")
    lines.append("")
    lines.append("## Recommendations")
    lines.append("")
    if report["recommendations"]:
        for item in report["recommendations"]:
            lines.append(f"- {item}")
    else:
        lines.append("- None.")
    lines.append("")
    return "\n".join(lines)


def write_report_files(workspace_root: Path, report: dict[str, Any], *, stem: str = "aoa_codex_hooks_report") -> dict[str, str]:
    report_dir = ensure_report_dir(workspace_root)
    json_path = report_dir / f"{stem}.json"
    md_path = report_dir / f"{stem}.md"
    latest_json = report_dir / f"{stem}_latest.json"
    latest_md = report_dir / f"{stem}_latest.md"

    payload = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    markdown = report_to_markdown(report)

    json_path.write_text(payload, encoding="utf-8")
    md_path.write_text(markdown, encoding="utf-8")
    latest_json.write_text(payload, encoding="utf-8")
    latest_md.write_text(markdown, encoding="utf-8")

    return {
        "json": str(json_path),
        "markdown": str(md_path),
        "latest_json": str(latest_json),
        "latest_markdown": str(latest_md),
    }


def prompt_mentions_codex_surfaces(prompt: str) -> bool:
    lowered = prompt.lower()
    return any(token in lowered for token in TRIGGER_WORDS)
