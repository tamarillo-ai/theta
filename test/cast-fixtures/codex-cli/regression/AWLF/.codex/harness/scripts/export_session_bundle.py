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
    RUN_ROOT,
    SCHEMA_ROOT,
    WORKSPACE_ROOT,
    bundle_artifact_path,
    configure_utf8_stdio,
    infer_completed_stages,
    load_json,
    validate_with_named_schema,
    write_json,
)


configure_utf8_stdio()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Export a real task workspace into a session bundle.")
    parser.add_argument("--task-root", type=Path, required=True, help="Path to one real task workspace.")
    parser.add_argument("--eval-id", help="Eval id. Overrides metadata.json when provided.")
    parser.add_argument("--bundle-name", help="Bundle name. Defaults to metadata or workspace directory name.")
    parser.add_argument(
        "--output-root",
        type=Path,
        default=RUN_ROOT,
        help="Directory where exported bundles are written. Default: .codex/harness/evals/runs",
    )
    parser.add_argument("--generated-at", help="Explicit manifest timestamp. Defaults to current local time.")
    parser.add_argument("--user-approved", action="store_true", help="Force user_approval_granted=true.")
    parser.add_argument("--approval-evidence", help="Explicit approval evidence text.")
    parser.add_argument("--overwrite", action="store_true", help="Overwrite output bundle when it exists.")
    parser.add_argument("--validate", action="store_true", help="Run session acceptance after export.")
    return parser.parse_args()


def discover_workspace(task_root: Path) -> tuple[dict[str, Any], Path, Path]:
    metadata_path = task_root / "metadata.json"
    transcript_path = task_root / "transcript.md"
    artifact_dir = task_root / "artifacts"

    metadata: dict[str, Any] = {}
    if metadata_path.exists():
        metadata = load_json(metadata_path)
        errors = validate_with_named_schema("metadata", metadata)
        if errors:
            raise ValueError(f"metadata.json invalid: {'; '.join(errors)}")

    if not transcript_path.exists():
        raise FileNotFoundError(f"missing transcript: {transcript_path}")
    if not artifact_dir.is_dir():
        raise FileNotFoundError(f"missing artifacts directory: {artifact_dir}")

    return metadata, transcript_path, artifact_dir


def infer_user_approval(metadata: dict[str, Any], artifact_data: dict[str, Any], args: argparse.Namespace) -> tuple[bool, str]:
    if args.user_approved:
        return True, args.approval_evidence or ""

    if "user_approval_granted" in metadata:
        return bool(metadata["user_approval_granted"]), args.approval_evidence or metadata.get("user_approval_evidence", "")

    design_packet = artifact_data.get("design_packet")
    if isinstance(design_packet, dict) and design_packet.get("approval_status") == "approved":
        return True, args.approval_evidence or metadata.get("user_approval_evidence", "")

    return False, args.approval_evidence or metadata.get("user_approval_evidence", "")


def export_bundle(args: argparse.Namespace) -> Path:
    task_root = args.task_root.resolve()
    metadata, transcript_path, artifact_dir = discover_workspace(task_root)

    artifact_paths = sorted(artifact_dir.glob("*.json"))
    if not artifact_paths:
        raise FileNotFoundError(f"no artifact json files found in {artifact_dir}")

    artifact_data: dict[str, Any] = {}
    for artifact_path in artifact_paths:
        artifact_name = artifact_path.stem
        data = load_json(artifact_path)
        errors = validate_with_named_schema(artifact_name, data)
        if errors:
            raise ValueError(f"{artifact_path.name} invalid: {'; '.join(errors)}")
        artifact_data[artifact_name] = data

    eval_id = args.eval_id or metadata.get("eval_id")
    if not eval_id:
        raise ValueError("eval id is required. Provide --eval-id or metadata.json.eval_id.")

    bundle_name = args.bundle_name or metadata.get("bundle_name") or task_root.name
    output_root = args.output_root.resolve()
    bundle_root = output_root / bundle_name

    if bundle_root.exists():
        if not args.overwrite:
            raise FileExistsError(f"bundle already exists: {bundle_root}. Use --overwrite to replace it.")
        shutil.rmtree(bundle_root)

    (bundle_root / "artifacts").mkdir(parents=True, exist_ok=True)

    user_approval_granted, user_approval_evidence = infer_user_approval(metadata, artifact_data, args)
    workflow_state = artifact_data.get("workflow_state", {})
    current_state = workflow_state.get("current_state") if isinstance(workflow_state, dict) else None
    completed_stages = metadata.get("completed_stages") or infer_completed_stages(
        set(artifact_data.keys()),
        current_state,
        user_approval_granted,
    )

    manifest = {
        "bundle_name": bundle_name,
        "eval_id": eval_id,
        "generated_at": args.generated_at or metadata.get("generated_at") or datetime.now().astimezone().isoformat(timespec="seconds"),
        "completed_stages": completed_stages,
        "user_approval_granted": user_approval_granted,
    }
    if user_approval_evidence:
        manifest["user_approval_evidence"] = user_approval_evidence
    if metadata.get("notes"):
        manifest["notes"] = metadata["notes"]

    manifest_errors = validate_with_named_schema("manifest", manifest)
    if manifest_errors:
        raise ValueError(f"generated manifest invalid: {'; '.join(manifest_errors)}")

    write_json(bundle_root / "manifest.json", manifest)
    shutil.copy2(transcript_path, bundle_root / "transcript.md")

    for artifact_path in artifact_paths:
        shutil.copy2(artifact_path, bundle_artifact_path(bundle_root, artifact_path.stem))

    print(f"Exported bundle: {bundle_root}", flush=True)
    print(f"Eval id: {eval_id}", flush=True)
    print(f"Completed stages: {', '.join(completed_stages)}", flush=True)
    return bundle_root


def maybe_validate(bundle_root: Path, eval_id: str) -> int:
    spec_path = HARNESS_ROOT / "evals" / "specs" / f"{eval_id}.json"
    if not spec_path.exists():
        print(f"Skip validation: spec not found for eval id {eval_id!r}", flush=True)
        return 0

    command = [
        sys.executable,
        str(HARNESS_ROOT / "scripts" / "run_session_acceptance.py"),
        "--bundle",
        str(bundle_root),
        "--spec",
        str(spec_path),
    ]
    print("Running acceptance validation...", flush=True)
    return subprocess.run(command, check=False).returncode


def main() -> int:
    args = parse_args()
    bundle_root = export_bundle(args)
    manifest = load_json(bundle_root / "manifest.json")
    if args.validate:
        return maybe_validate(bundle_root, manifest["eval_id"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
