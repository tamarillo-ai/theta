from __future__ import annotations

import json
import sys
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1] / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from aoa_codex_converge.convergence_state import build_report, report_to_markdown, write_bootstrap_scaffold


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def _minimal_config(root: Path) -> str:
    return f"""
project_root_markers = ["AOA_WORKSPACE_ROOT", ".git"]

[mcp_servers.aoa_workspace]
command = "python"
args = ["scripts/aoa_workspace_mcp_server.py"]
cwd = "{root / 'aoa-sdk'}"

[mcp_servers.aoa_stats]
command = "python"
args = ["scripts/aoa_stats_mcp_server.py"]
cwd = "{root / 'aoa-stats'}"

[mcp_servers.dionysus]
command = "python"
args = ["scripts/dionysus_mcp_server.py"]
cwd = "{root / 'Dionysus'}"
"""


def _agent_toml(name: str) -> str:
    return f"""
name = "{name}"
description = "test"
developer_instructions = "test"
"""


def _plugin_manifest() -> dict[str, object]:
    return {
        "name": "aoa-shared-launchers",
        "version": "0.1.0",
        "description": "test plugin",
        "skills": "./skills/",
    }


def test_write_bootstrap_scaffold_creates_core_paths(tmp_path: Path) -> None:
    created = write_bootstrap_scaffold(tmp_path)
    assert (tmp_path / "AOA_WORKSPACE_ROOT").exists()
    assert (tmp_path / ".codex" / "agents").exists()
    assert (tmp_path / ".codex" / "bin" / "aoa-codex-doctor").exists()
    assert (tmp_path / ".codex" / "generated" / "codex" / "config.convergence.generated.toml").exists()
    assert str(tmp_path / "AOA_WORKSPACE_ROOT") in created


def test_ready_report_when_required_seams_exist(tmp_path: Path) -> None:
    write_bootstrap_scaffold(tmp_path)
    _write(tmp_path / ".codex" / "config.toml", _minimal_config(tmp_path))
    _write(tmp_path / "aoa-sdk" / "scripts" / "aoa_workspace_mcp_server.py", "print('ok')\n")
    for name in ["architect", "coder", "reviewer", "evaluator", "memory-keeper"]:
        _write(tmp_path / ".codex" / "agents" / f"{name}.toml", _agent_toml(name))

    report = build_report(tmp_path)
    assert report["ready"] is True
    names_to_status = {item["name"]: item["status"] for item in report["surfaces"]}
    assert names_to_status["workspace_root_anchor"] == "ok"
    assert names_to_status["workspace_mcp"] == "ok"
    assert names_to_status["subagents_surface"] == "ok"


def test_warn_when_marker_not_listed_in_config(tmp_path: Path) -> None:
    write_bootstrap_scaffold(tmp_path)
    _write(
        tmp_path / ".codex" / "config.toml",
        f"""
project_root_markers = [".git"]

[mcp_servers.aoa_workspace]
command = "python"
args = ["scripts/aoa_workspace_mcp_server.py"]
cwd = "{tmp_path / 'aoa-sdk'}"
""",
    )
    _write(tmp_path / "aoa-sdk" / "scripts" / "aoa_workspace_mcp_server.py", "print('ok')\n")
    for name in ["architect", "coder", "reviewer", "evaluator", "memory-keeper"]:
        _write(tmp_path / ".codex" / "agents" / f"{name}.toml", _agent_toml(name))

    report = build_report(tmp_path)
    item = next(s for s in report["surfaces"] if s["name"] == "project_root_marker_config")
    assert item["status"] == "warn"
    assert report["ready"] is False


def test_optional_repo_mcp_detection(tmp_path: Path) -> None:
    write_bootstrap_scaffold(tmp_path)
    _write(tmp_path / ".codex" / "config.toml", _minimal_config(tmp_path))
    _write(tmp_path / "aoa-sdk" / "scripts" / "aoa_workspace_mcp_server.py", "print('ok')\n")
    for name in ["architect", "coder", "reviewer", "evaluator", "memory-keeper"]:
        _write(tmp_path / ".codex" / "agents" / f"{name}.toml", _agent_toml(name))
    _write(tmp_path / "aoa-stats" / "scripts" / "aoa_stats_mcp_server.py", "print('ok')\n")
    _write_json(tmp_path / "aoa-stats" / "generated" / "summary_surface_catalog.min.json", {"surfaces": []})
    _write(tmp_path / "Dionysus" / "scripts" / "dionysus_mcp_server.py", "print('ok')\n")
    _write_json(tmp_path / "Dionysus" / "generated" / "seed_route_map.min.json", {"routes": []})

    report = build_report(tmp_path)
    names_to_status = {item["name"]: item["status"] for item in report["surfaces"]}
    assert names_to_status["stats_mcp"] == "ok"
    assert names_to_status["dionysus_mcp"] == "ok"


def test_plugin_marketplace_path_can_point_into_dot_codex(tmp_path: Path) -> None:
    write_bootstrap_scaffold(tmp_path)
    _write(tmp_path / ".codex" / "config.toml", _minimal_config(tmp_path))
    _write(tmp_path / "aoa-sdk" / "scripts" / "aoa_workspace_mcp_server.py", "print('ok')\n")
    for name in ["architect", "coder", "reviewer", "evaluator", "memory-keeper"]:
        _write(tmp_path / ".codex" / "agents" / f"{name}.toml", _agent_toml(name))
    _write_json(
        tmp_path / ".agents" / "plugins" / "marketplace.json",
        {
            "name": "aoa-local-plugins",
            "plugins": [
                {
                    "name": "aoa-shared-launchers",
                    "source": {
                        "source": "local",
                        "path": "./.codex/plugins/aoa-shared-launchers",
                    },
                }
            ],
        },
    )
    _write_json(
        tmp_path / ".codex" / "plugins" / "aoa-shared-launchers" / ".codex-plugin" / "plugin.json",
        _plugin_manifest(),
    )

    report = build_report(tmp_path)
    names_to_status = {item["name"]: item["status"] for item in report["surfaces"]}
    assert names_to_status["plugin_marketplace"] == "ok"
    assert names_to_status["plugin_source"] == "ok"
    assert names_to_status["skill_bridge"] == "ok"


def test_warn_for_stranded_sibling_skills(tmp_path: Path) -> None:
    write_bootstrap_scaffold(tmp_path)
    _write(
        tmp_path / ".codex" / "config.toml",
        f"""
project_root_markers = ["AOA_WORKSPACE_ROOT", ".git"]

[mcp_servers.aoa_workspace]
command = "python"
args = ["scripts/aoa_workspace_mcp_server.py"]
cwd = "{tmp_path / 'aoa-sdk'}"
""",
    )
    _write(tmp_path / "aoa-sdk" / "scripts" / "aoa_workspace_mcp_server.py", "print('ok')\n")
    for name in ["architect", "coder", "reviewer", "evaluator", "memory-keeper"]:
        _write(tmp_path / ".codex" / "agents" / f"{name}.toml", _agent_toml(name))
    _write(
        tmp_path / "aoa-skills" / ".agents" / "skills" / "dummy" / "SKILL.md",
        "---\nname: dummy\ndescription: dummy\n---\n",
    )

    report = build_report(tmp_path)
    item = next(s for s in report["surfaces"] if s["name"] == "skill_bridge")
    assert item["status"] == "warn"
    assert "stranded" in item["summary"]


def test_markdown_renderer_mentions_counts(tmp_path: Path) -> None:
    write_bootstrap_scaffold(tmp_path)
    report = build_report(tmp_path)
    text = report_to_markdown(report)
    assert "# AoA Codex convergence report" in text
    assert "counts:" in text
