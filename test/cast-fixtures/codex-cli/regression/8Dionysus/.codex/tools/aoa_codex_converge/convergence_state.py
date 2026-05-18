from __future__ import annotations

import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Mapping

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


EXPECTED_SUBAGENTS = [
    "architect",
    "coder",
    "reviewer",
    "evaluator",
    "memory-keeper",
]
EXPECTED_PLUGIN_NAME = "aoa-shared-launchers"


@dataclass
class SurfaceStatus:
    name: str
    status: str
    required: bool
    summary: str
    evidence: list[str]
    next_step: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def _read_toml(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    with path.open("rb") as fh:
        return tomllib.load(fh)


def _read_json(path: Path) -> Any:
    if not path.exists():
        return None
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def _normalize_workspace_root(workspace_root: Path | str) -> Path:
    return Path(workspace_root).expanduser().resolve()


def _collect_toml_names(agents_dir: Path) -> list[str]:
    names: list[str] = []
    if not agents_dir.exists():
        return names
    for path in sorted(agents_dir.glob("*.toml")):
        try:
            data = _read_toml(path)
        except Exception:
            continue
        name = data.get("name")
        if isinstance(name, str) and name.strip():
            names.append(name.strip())
    return names


def _marketplace_root(marketplace_path: Path) -> Path:
    parts = marketplace_path.parts
    if len(parts) >= 3 and parts[-3:] == (".agents", "plugins", "marketplace.json"):
        return marketplace_path.parents[2]
    return marketplace_path.parent


def _plugin_entry(marketplace: Any, plugin_name: str) -> Mapping[str, Any] | None:
    if not isinstance(marketplace, dict):
        return None
    plugins = marketplace.get("plugins", [])
    if not isinstance(plugins, list):
        return None
    for item in plugins:
        if isinstance(item, dict) and item.get("name") == plugin_name:
            return item
    return None


def _resolve_plugin_root(
    workspace_root: Path,
    marketplace_path: Path,
    marketplace: Any,
    plugin_name: str,
) -> tuple[Path | None, list[str]]:
    evidence: list[str] = []
    entry = _plugin_entry(marketplace, plugin_name)
    if entry is not None:
        evidence.append(f"marketplace.plugin={plugin_name}")
        source = entry.get("source", {})
        source_path = source.get("path") if isinstance(source, dict) else None
        if isinstance(source_path, str) and source_path.startswith("./"):
            marketplace_root = _marketplace_root(marketplace_path.resolve())
            resolved = (marketplace_root / source_path).resolve()
            evidence.append(f"marketplace.source.path={source_path}")
            evidence.append(str(resolved))
            return resolved, evidence

    for candidate in (
        workspace_root / ".codex" / "plugins" / plugin_name,
        workspace_root / "plugins" / plugin_name,
    ):
        if candidate.exists():
            evidence.append(str(candidate))
            return candidate, evidence
    return None, evidence


def _resolve_mcp_script(
    workspace_root: Path,
    entry: object,
) -> tuple[Path | None, list[str]]:
    evidence: list[str] = []
    if not isinstance(entry, Mapping):
        return None, evidence

    command = entry.get("command")
    if isinstance(command, str) and command:
        evidence.append(f"command={command}")

    base = workspace_root
    cwd = entry.get("cwd")
    if isinstance(cwd, str) and cwd:
        base = Path(cwd).expanduser()
        if not base.is_absolute():
            base = (workspace_root / base).resolve()
        else:
            base = base.resolve()
        evidence.append(f"cwd={base}")

    args = entry.get("args")
    if isinstance(args, list):
        evidence.append(f"args={args!r}")
        for arg in args:
            if isinstance(arg, str) and arg.endswith(".py"):
                script = Path(arg)
                if not script.is_absolute():
                    script = (base / script).resolve()
                else:
                    script = script.resolve()
                return script, evidence

    return None, evidence


def build_report(workspace_root: Path | str) -> dict[str, Any]:
    root = _normalize_workspace_root(workspace_root)

    marker_path = root / "AOA_WORKSPACE_ROOT"
    codex_config_path = root / ".codex" / "config.toml"
    agents_dir = root / ".codex" / "agents"
    convergence_scripts_dir = root / ".codex" / "scripts"
    convergence_bin_dir = root / ".codex" / "bin"
    plugin_marketplace_path = root / ".agents" / "plugins" / "marketplace.json"
    workspace_skill_dir = root / ".agents" / "skills"
    sibling_aoa_skills_dir = root / "aoa-skills" / ".agents" / "skills"
    stats_repo = root / "aoa-stats"
    stats_catalog = stats_repo / "generated" / "summary_surface_catalog.min.json"
    dionysus_repo = root / "Dionysus"
    dionysus_catalog = dionysus_repo / "generated" / "seed_route_map.min.json"

    config = _read_toml(codex_config_path)
    marketplace = _read_json(plugin_marketplace_path)
    subagent_names = _collect_toml_names(agents_dir)
    mcp_servers = config.get("mcp_servers", {}) if isinstance(config, dict) else {}
    project_root_markers = config.get("project_root_markers", []) if isinstance(config, dict) else []

    surfaces: list[SurfaceStatus] = []

    marker_exists = marker_path.exists()
    surfaces.append(
        SurfaceStatus(
            name="workspace_root_anchor",
            status="ok" if marker_exists else "missing",
            required=True,
            summary="Workspace root marker is present." if marker_exists else "Missing AOA_WORKSPACE_ROOT marker.",
            evidence=[str(marker_path)] if marker_exists else [],
            next_step="Create AOA_WORKSPACE_ROOT at the intended sibling workspace root.",
        )
    )

    config_exists = codex_config_path.exists()
    surfaces.append(
        SurfaceStatus(
            name="codex_project_config",
            status="ok" if config_exists else "missing",
            required=True,
            summary="Workspace .codex/config.toml exists." if config_exists else "Missing workspace .codex/config.toml.",
            evidence=[str(codex_config_path)] if config_exists else [],
            next_step="Create or merge a trusted project-scoped .codex/config.toml at the workspace root.",
        )
    )

    marker_listed = isinstance(project_root_markers, list) and "AOA_WORKSPACE_ROOT" in project_root_markers
    surfaces.append(
        SurfaceStatus(
            name="project_root_marker_config",
            status="ok" if marker_listed else ("warn" if config_exists else "missing"),
            required=True,
            summary="project_root_markers includes AOA_WORKSPACE_ROOT."
            if marker_listed
            else "project_root_markers does not include AOA_WORKSPACE_ROOT.",
            evidence=[f"project_root_markers={project_root_markers!r}"] if config_exists else [],
            next_step="Add AOA_WORKSPACE_ROOT to project_root_markers so sibling repos do not steal project-root discovery.",
        )
    )

    workspace_entry = mcp_servers.get("aoa_workspace") if isinstance(mcp_servers, dict) else None
    workspace_script_path, workspace_mcp_evidence = _resolve_mcp_script(root, workspace_entry)
    if isinstance(mcp_servers, dict) and "aoa_workspace" in mcp_servers:
        workspace_mcp_evidence.insert(0, "[mcp_servers.aoa_workspace] in .codex/config.toml")
    if workspace_script_path is not None and workspace_script_path.exists():
        workspace_mcp_evidence.append(str(workspace_script_path))
    workspace_mcp_ok = (
        isinstance(mcp_servers, dict)
        and "aoa_workspace" in mcp_servers
        and workspace_script_path is not None
        and workspace_script_path.exists()
    )
    surfaces.append(
        SurfaceStatus(
            name="workspace_mcp",
            status="ok" if workspace_mcp_ok else "missing",
            required=True,
            summary="Workspace MCP is configured and resolves to a live script."
            if workspace_mcp_ok
            else "Workspace MCP is not fully converged.",
            evidence=workspace_mcp_evidence,
            next_step="Ensure [mcp_servers.aoa_workspace] exists and its cwd/args resolve to a live aoa_workspace MCP server script.",
        )
    )

    stats_repo_exists = stats_repo.exists()
    stats_entry = mcp_servers.get("aoa_stats") if isinstance(mcp_servers, dict) else None
    stats_script_path, stats_evidence = _resolve_mcp_script(root, stats_entry)
    if stats_repo_exists:
        stats_evidence.insert(0, str(stats_repo))
    if isinstance(mcp_servers, dict) and "aoa_stats" in mcp_servers:
        stats_evidence.insert(1 if stats_repo_exists else 0, "[mcp_servers.aoa_stats] in .codex/config.toml")
    if stats_script_path is not None and stats_script_path.exists():
        stats_evidence.append(str(stats_script_path))
    if stats_catalog.exists():
        stats_evidence.append(str(stats_catalog))
    stats_mcp_ok = (
        not stats_repo_exists
        or (
            isinstance(mcp_servers, dict)
            and "aoa_stats" in mcp_servers
            and stats_script_path is not None
            and stats_script_path.exists()
            and stats_catalog.exists()
        )
    )
    surfaces.append(
        SurfaceStatus(
            name="stats_mcp",
            status="info" if not stats_repo_exists else ("ok" if stats_mcp_ok else "warn"),
            required=False,
            summary=(
                "aoa-stats repo not present under the workspace root."
                if not stats_repo_exists
                else ("aoa-stats MCP surface looks wired." if stats_mcp_ok else "aoa-stats repo exists but its MCP seam is incomplete.")
            ),
            evidence=stats_evidence,
            next_step="Wire aoa_stats into workspace .codex/config.toml and ensure the repo-local server plus generated catalog exist.",
        )
    )

    dionysus_repo_exists = dionysus_repo.exists()
    dionysus_entry = mcp_servers.get("dionysus") if isinstance(mcp_servers, dict) else None
    dionysus_script_path, dionysus_evidence = _resolve_mcp_script(root, dionysus_entry)
    if dionysus_repo_exists:
        dionysus_evidence.insert(0, str(dionysus_repo))
    if isinstance(mcp_servers, dict) and "dionysus" in mcp_servers:
        dionysus_evidence.insert(
            1 if dionysus_repo_exists else 0,
            "[mcp_servers.dionysus] in .codex/config.toml",
        )
    if dionysus_script_path is not None and dionysus_script_path.exists():
        dionysus_evidence.append(str(dionysus_script_path))
    if dionysus_catalog.exists():
        dionysus_evidence.append(str(dionysus_catalog))
    dionysus_mcp_ok = (
        not dionysus_repo_exists
        or (
            isinstance(mcp_servers, dict)
            and "dionysus" in mcp_servers
            and dionysus_script_path is not None
            and dionysus_script_path.exists()
            and dionysus_catalog.exists()
        )
    )
    surfaces.append(
        SurfaceStatus(
            name="dionysus_mcp",
            status="info" if not dionysus_repo_exists else ("ok" if dionysus_mcp_ok else "warn"),
            required=False,
            summary=(
                "Dionysus repo not present under the workspace root."
                if not dionysus_repo_exists
                else ("Dionysus MCP surface looks wired." if dionysus_mcp_ok else "Dionysus repo exists but its MCP seam is incomplete.")
            ),
            evidence=dionysus_evidence,
            next_step="Wire dionysus into workspace .codex/config.toml and ensure the repo-local server plus generated seed route map exist.",
        )
    )

    missing_subagents = [name for name in EXPECTED_SUBAGENTS if name not in subagent_names]
    subagents_ok = not missing_subagents and agents_dir.exists()
    subagent_evidence = [str(agents_dir)] if agents_dir.exists() else []
    subagent_evidence.extend(f"name={name}" for name in subagent_names)
    surfaces.append(
        SurfaceStatus(
            name="subagents_surface",
            status="ok" if subagents_ok else "warn",
            required=True,
            summary="Expected AoA subagents are present." if subagents_ok else "Some expected AoA subagents are missing.",
            evidence=subagent_evidence,
            next_step="Generate or copy the AoA subagent TOML files into .codex/agents/ and register them in workspace config if needed.",
        )
    )

    plugin_names: list[str] = []
    if isinstance(marketplace, dict):
        plugins = marketplace.get("plugins", [])
        if isinstance(plugins, list):
            for item in plugins:
                if isinstance(item, dict):
                    name = item.get("name")
                    if isinstance(name, str) and name:
                        plugin_names.append(name)
    plugin_marketplace_ok = plugin_marketplace_path.exists() and EXPECTED_PLUGIN_NAME in plugin_names
    plugin_marketplace_evidence = [str(plugin_marketplace_path)] if plugin_marketplace_path.exists() else []
    plugin_marketplace_evidence.extend(f"plugin={name}" for name in plugin_names)
    surfaces.append(
        SurfaceStatus(
            name="plugin_marketplace",
            status="ok" if plugin_marketplace_ok else "warn",
            required=False,
            summary="Workspace marketplace exposes aoa-shared-launchers."
            if plugin_marketplace_ok
            else "Workspace marketplace is missing or does not expose aoa-shared-launchers.",
            evidence=plugin_marketplace_evidence,
            next_step="Create .agents/plugins/marketplace.json or add aoa-shared-launchers to the workspace marketplace.",
        )
    )

    plugin_root, plugin_source_evidence = _resolve_plugin_root(
        root,
        plugin_marketplace_path,
        marketplace,
        EXPECTED_PLUGIN_NAME,
    )
    plugin_manifest_path = plugin_root / ".codex-plugin" / "plugin.json" if plugin_root is not None else None
    plugin_manifest = _read_json(plugin_manifest_path) if plugin_manifest_path is not None else None
    plugin_manifest_ok = plugin_manifest_path is not None and plugin_manifest_path.exists() and isinstance(plugin_manifest, dict)
    if plugin_manifest_ok:
        plugin_source_evidence.append(str(plugin_manifest_path))
        plugin_name = plugin_manifest.get("name")
        if isinstance(plugin_name, str) and plugin_name:
            plugin_source_evidence.append(f"plugin.name={plugin_name}")
    surfaces.append(
        SurfaceStatus(
            name="plugin_source",
            status="ok" if plugin_manifest_ok else "warn",
            required=False,
            summary="Local shared plugin source exists." if plugin_manifest_ok else "Local shared plugin source is missing.",
            evidence=plugin_source_evidence,
            next_step="Place the aoa-shared-launchers plugin under .codex/plugins/ or point a marketplace entry at its true location.",
        )
    )

    workspace_skill_exists = workspace_skill_dir.exists() and any(workspace_skill_dir.glob("*"))
    sibling_skill_exists = sibling_aoa_skills_dir.exists() and any(sibling_aoa_skills_dir.glob("*"))
    skill_bridge_ok = workspace_skill_exists or plugin_marketplace_ok or plugin_manifest_ok
    skill_evidence: list[str] = []
    if workspace_skill_exists:
        skill_evidence.append(str(workspace_skill_dir))
    if sibling_skill_exists:
        skill_evidence.append(str(sibling_aoa_skills_dir))
    if plugin_marketplace_ok:
        skill_evidence.append("workspace-visible skills may arrive through the aoa-shared-launchers plugin")
    if plugin_manifest_ok:
        skill_evidence.append("local plugin source is available for installation")
    if skill_bridge_ok:
        skill_status = "ok"
        skill_summary = "Workspace-visible skill bridge is present."
    elif sibling_skill_exists:
        skill_status = "warn"
        skill_summary = "Skills appear stranded in a sibling repo without a workspace-visible bridge."
    else:
        skill_status = "warn"
        skill_summary = "No obvious workspace-visible skill bridge was found."
    surfaces.append(
        SurfaceStatus(
            name="skill_bridge",
            status=skill_status,
            required=False,
            summary=skill_summary,
            evidence=skill_evidence,
            next_step="Expose shared skills through a plugin or a workspace-visible .agents/skills bridge instead of relying on sibling-repo discovery.",
        )
    )

    convergence_tooling_evidence: list[str] = []
    tooling_ok = True
    for path in (
        convergence_scripts_dir / "aoa_codex_bootstrap.py",
        convergence_scripts_dir / "aoa_codex_doctor.py",
        convergence_scripts_dir / "aoa_codex_status.py",
        convergence_bin_dir / "aoa-codex-bootstrap",
        convergence_bin_dir / "aoa-codex-doctor",
        convergence_bin_dir / "aoa-codex-status",
    ):
        if path.exists():
            convergence_tooling_evidence.append(str(path))
        else:
            tooling_ok = False
    surfaces.append(
        SurfaceStatus(
            name="convergence_tooling",
            status="ok" if tooling_ok else "warn",
            required=False,
            summary="Convergence tooling is installed under .codex."
            if tooling_ok
            else "Convergence tooling is only partially installed under .codex.",
            evidence=convergence_tooling_evidence,
            next_step="Install the aoa_codex_converge package plus the .codex/scripts and .codex/bin wrappers.",
        )
    )

    counts = {"ok": 0, "warn": 0, "missing": 0, "info": 0}
    for surface in surfaces:
        counts[surface.status] = counts.get(surface.status, 0) + 1

    ready = not any(surface.required and surface.status != "ok" for surface in surfaces)
    return {
        "workspace_root": str(root),
        "ready": ready,
        "counts": counts,
        "surfaces": [surface.to_dict() for surface in surfaces],
    }


def report_to_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# AoA Codex convergence report",
        "",
        f"- workspace_root: `{report.get('workspace_root', '')}`",
        f"- ready: `{report.get('ready', False)}`",
    ]
    counts = report.get("counts", {})
    lines.append(
        f"- counts: ok={counts.get('ok', 0)}, warn={counts.get('warn', 0)}, "
        f"missing={counts.get('missing', 0)}, info={counts.get('info', 0)}"
    )
    lines.append("")
    lines.append("## Surfaces")
    lines.append("")
    for surface in report.get("surfaces", []):
        lines.append(f"### {surface.get('name', '')}")
        lines.append("")
        lines.append(f"- status: `{surface.get('status', '')}`")
        lines.append(f"- required: `{surface.get('required', False)}`")
        lines.append(f"- summary: {surface.get('summary', '')}")
        evidence = surface.get("evidence", [])
        if evidence:
            lines.append("- evidence:")
            for item in evidence:
                lines.append(f"  - `{item}`")
        else:
            lines.append("- evidence: none")
        lines.append(f"- next_step: {surface.get('next_step', '')}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


BOOTSTRAP_CONFIG_SNIPPET = """# Review-only convergence snippet.
# Merge this into /srv/AbyssOS/.codex/config.toml only if the marker entry is missing.
project_root_markers = ["AOA_WORKSPACE_ROOT", ".git"]
"""

BOOTSTRAP_WRAPPER_DOCTOR = """#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

python "${WORKSPACE_ROOT}/.codex/scripts/aoa_codex_doctor.py" --workspace-root "${WORKSPACE_ROOT}" "$@"
"""

BOOTSTRAP_WRAPPER_STATUS = """#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

python "${WORKSPACE_ROOT}/.codex/scripts/aoa_codex_status.py" --workspace-root "${WORKSPACE_ROOT}" "$@"
"""

BOOTSTRAP_WRAPPER_BOOTSTRAP = """#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

python "${WORKSPACE_ROOT}/.codex/scripts/aoa_codex_bootstrap.py" --workspace-root "${WORKSPACE_ROOT}" "$@"
"""


def write_bootstrap_scaffold(workspace_root: Path | str) -> dict[str, str]:
    root = _normalize_workspace_root(workspace_root)
    created: dict[str, str] = {}

    directories = [
        root / ".codex",
        root / ".codex" / "agents",
        root / ".codex" / "bin",
        root / ".codex" / "generated",
        root / ".codex" / "generated" / "codex",
        root / ".codex" / "plugins",
        root / ".codex" / "scripts",
        root / ".codex" / "tools",
        root / ".agents",
        root / ".agents" / "plugins",
        root / ".agents" / "skills",
    ]
    for path in directories:
        path.mkdir(parents=True, exist_ok=True)
        created[str(path)] = "dir"

    marker = root / "AOA_WORKSPACE_ROOT"
    if not marker.exists():
        marker.write_text("AoA sibling workspace root marker.\n", encoding="utf-8")
        created[str(marker)] = "file"

    config_snippet = root / ".codex" / "generated" / "codex" / "config.convergence.generated.toml"
    config_snippet.write_text(BOOTSTRAP_CONFIG_SNIPPET, encoding="utf-8")
    created[str(config_snippet)] = "file"

    wrapper_specs = {
        root / ".codex" / "bin" / "aoa-codex-doctor": BOOTSTRAP_WRAPPER_DOCTOR,
        root / ".codex" / "bin" / "aoa-codex-status": BOOTSTRAP_WRAPPER_STATUS,
        root / ".codex" / "bin" / "aoa-codex-bootstrap": BOOTSTRAP_WRAPPER_BOOTSTRAP,
    }
    for path, text in wrapper_specs.items():
        path.write_text(text, encoding="utf-8")
        os.chmod(path, 0o755)
        created[str(path)] = "file"

    return created
