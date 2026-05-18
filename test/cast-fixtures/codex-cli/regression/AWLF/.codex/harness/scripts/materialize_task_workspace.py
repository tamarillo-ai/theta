from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

from harness_utils import (
    HARNESS_ROOT,
    METADATA_CANDIDATE_NAMES,
    TRANSCRIPT_CANDIDATE_NAMES,
    WORKSPACE_ARTIFACT_NAMES,
    WORKSPACE_ROOT,
    configure_utf8_stdio,
    infer_completed_stages,
    load_json,
    validate_with_named_schema,
    write_json,
)


configure_utf8_stdio()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Materialize a task workspace from existing task materials.")
    parser.add_argument("--source-root", type=Path, required=True, help="Source directory: bundle, workspace, or loose materials.")
    parser.add_argument("--workspace-name", help="Output workspace name. Defaults to source directory name.")
    parser.add_argument(
        "--output-root",
        type=Path,
        default=WORKSPACE_ROOT,
        help="Directory where workspaces are written. Default: .codex/harness/evals/workspaces",
    )
    parser.add_argument("--task-id", help="Override task id.")
    parser.add_argument("--eval-id", help="Override eval id.")
    parser.add_argument("--generated-at", help="Override metadata timestamp.")
    parser.add_argument("--transcript", type=Path, help="Explicit transcript markdown path.")
    parser.add_argument("--metadata", type=Path, help="Explicit metadata json path.")
    parser.add_argument("--artifacts-root", type=Path, help="Explicit artifacts directory for loose materials mode.")
    parser.add_argument("--overwrite", action="store_true", help="Overwrite output workspace when it exists.")
    parser.add_argument("--export-bundle", action="store_true", help="After workspace materialization, export a bundle.")
    parser.add_argument("--validate", action="store_true", help="When exporting a bundle, also run acceptance validation.")
    return parser.parse_args()


def detect_source_mode(source_root: Path) -> str:
    if (source_root / "manifest.json").exists() and (source_root / "artifacts").is_dir():
        return "bundle"
    if (source_root / "transcript.md").exists() and (source_root / "artifacts").is_dir():
        return "workspace"
    return "materials"


def find_transcript(source_root: Path, explicit_path: Path | None) -> Path:
    candidates: list[Path] = []
    if explicit_path:
        candidates.append(explicit_path)
    else:
        for name in TRANSCRIPT_CANDIDATE_NAMES:
            direct = source_root / name
            if direct.exists():
                candidates.append(direct)
        if not candidates:
            candidates.extend(path for path in source_root.rglob("*.md") if path.name in TRANSCRIPT_CANDIDATE_NAMES)

    for candidate in candidates:
        resolved = candidate if candidate.is_absolute() else candidate.resolve()
        if resolved.exists():
            return resolved
    raise FileNotFoundError("Unable to find transcript markdown file.")


def find_metadata(source_root: Path, explicit_path: Path | None) -> dict[str, Any]:
    candidate_paths: list[Path] = []
    if explicit_path:
        candidate_paths.append(explicit_path)
    else:
        for name in METADATA_CANDIDATE_NAMES:
            direct = source_root / name
            if direct.exists():
                candidate_paths.append(direct)
        if not candidate_paths:
            candidate_paths.extend(path for path in source_root.rglob("*.json") if path.name in METADATA_CANDIDATE_NAMES)

    for candidate in candidate_paths:
        resolved = candidate if candidate.is_absolute() else candidate.resolve()
        if resolved.exists():
            data = load_json(resolved)
            errors = validate_with_named_schema("metadata", data)
            if errors:
                raise ValueError(f"{resolved.name} invalid: {'; '.join(errors)}")
            return data
    return {}


def collect_workspace_artifacts(source_root: Path, mode: str, explicit_artifacts_root: Path | None) -> dict[str, Path]:
    if mode in {"bundle", "workspace"}:
        artifact_dir = source_root / "artifacts"
        artifact_paths = sorted(artifact_dir.glob("*.json"))
        return {path.stem: path.resolve() for path in artifact_paths}

    search_root = explicit_artifacts_root.resolve() if explicit_artifacts_root else source_root.resolve()
    found: dict[str, Path] = {}
    for path in sorted(search_root.rglob("*.json")):
        if path.name in {"manifest.json", "metadata.json", "task-metadata.json", "workspace.json"}:
            continue
        if path.stem not in WORKSPACE_ARTIFACT_NAMES:
            continue
        if path.stem in found and found[path.stem] != path.resolve():
            raise ValueError(f"Duplicate artifact {path.stem!r}: {found[path.stem]} and {path.resolve()}")
        found[path.stem] = path.resolve()
    if not found:
        raise FileNotFoundError(f"No recognized artifacts found under {search_root}")
    return found


def validate_artifacts(artifacts: dict[str, Path]) -> None:
    for name, path in artifacts.items():
        errors = validate_with_named_schema(name, load_json(path))
        if errors:
            raise ValueError(f"{path.name} invalid: {'; '.join(errors)}")


def load_bundle_manifest(source_root: Path) -> dict[str, Any]:
    manifest_path = source_root / "manifest.json"
    if not manifest_path.exists():
        return {}
    manifest = load_json(manifest_path)
    errors = validate_with_named_schema("manifest", manifest)
    if errors:
        raise ValueError(f"manifest.json invalid: {'; '.join(errors)}")
    return manifest


def materialize_metadata(
    args: argparse.Namespace,
    source_root: Path,
    mode: str,
    artifact_paths: dict[str, Path],
    transcript_path: Path,
) -> dict[str, Any]:
    metadata = find_metadata(source_root, args.metadata)
    manifest = load_bundle_manifest(source_root) if mode == "bundle" else {}

    workflow_state: dict[str, Any] = {}
    if "workflow_state" in artifact_paths:
        workflow_state = load_json(artifact_paths["workflow_state"])

    design_packet: dict[str, Any] = {}
    if "design_packet" in artifact_paths:
        design_packet = load_json(artifact_paths["design_packet"])

    current_state = workflow_state.get("current_state")
    user_approval_granted = bool(
        metadata.get("user_approval_granted")
        if "user_approval_granted" in metadata
        else manifest.get("user_approval_granted")
        if "user_approval_granted" in manifest
        else design_packet.get("approval_status") == "approved"
    )
    user_approval_evidence = str(
        metadata.get("user_approval_evidence")
        or manifest.get("user_approval_evidence")
        or ""
    )

    completed_stages = (
        metadata.get("completed_stages")
        or manifest.get("completed_stages")
        or infer_completed_stages(set(artifact_paths.keys()), current_state, user_approval_granted)
    )

    task_id = args.task_id or metadata.get("task_id") or workflow_state.get("task_id") or source_root.name
    eval_id = args.eval_id or metadata.get("eval_id") or manifest.get("eval_id")
    bundle_name = metadata.get("bundle_name") or manifest.get("bundle_name")
    generated_at = (
        args.generated_at
        or metadata.get("generated_at")
        or manifest.get("generated_at")
        or datetime.now().astimezone().isoformat(timespec="seconds")
    )

    notes = list(metadata.get("notes", []))
    notes.append(f"Materialized from {mode} source.")
    notes.append(f"Transcript source: {transcript_path.name}")

    materialized = {
        "task_id": task_id,
        "generated_at": generated_at,
        "source_mode": mode,
        "source_root": str(source_root),
        "completed_stages": completed_stages,
        "user_approval_granted": user_approval_granted,
        "notes": notes,
    }
    if eval_id:
        materialized["eval_id"] = eval_id
    if bundle_name:
        materialized["bundle_name"] = bundle_name
    if user_approval_evidence:
        materialized["user_approval_evidence"] = user_approval_evidence

    errors = validate_with_named_schema("metadata", materialized)
    if errors:
        raise ValueError(f"generated metadata invalid: {'; '.join(errors)}")
    return materialized


def materialize_workspace(args: argparse.Namespace) -> Path:
    source_root = args.source_root.resolve()
    mode = detect_source_mode(source_root)
    transcript_path = find_transcript(source_root, args.transcript)
    artifact_paths = collect_workspace_artifacts(source_root, mode, args.artifacts_root)
    validate_artifacts(artifact_paths)
    metadata = materialize_metadata(args, source_root, mode, artifact_paths, transcript_path)

    workspace_name = args.workspace_name or source_root.name
    output_root = args.output_root.resolve()
    workspace_root = output_root / workspace_name

    if workspace_root.exists():
        if not args.overwrite:
            raise FileExistsError(f"workspace already exists: {workspace_root}. Use --overwrite to replace it.")
        shutil.rmtree(workspace_root)

    (workspace_root / "artifacts").mkdir(parents=True, exist_ok=True)
    write_json(workspace_root / "metadata.json", metadata)
    shutil.copy2(transcript_path, workspace_root / "transcript.md")

    for name, path in artifact_paths.items():
        shutil.copy2(path, workspace_root / "artifacts" / f"{name}.json")

    print(f"Materialized workspace: {workspace_root}", flush=True)
    print(f"Source mode: {mode}", flush=True)
    print(f"Stages: {', '.join(metadata['completed_stages'])}", flush=True)
    return workspace_root


def maybe_export_bundle(workspace_root: Path, validate: bool) -> int:
    command = [
        sys.executable,
        str(HARNESS_ROOT / "scripts" / "export_session_bundle.py"),
        "--task-root",
        str(workspace_root),
        "--overwrite",
    ]
    if validate:
        command.append("--validate")
    print("Exporting bundle from workspace...", flush=True)
    return subprocess.run(command, check=False).returncode


def main() -> int:
    args = parse_args()
    workspace_root = materialize_workspace(args)
    if args.export_bundle:
        return maybe_export_bundle(workspace_root, args.validate)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
