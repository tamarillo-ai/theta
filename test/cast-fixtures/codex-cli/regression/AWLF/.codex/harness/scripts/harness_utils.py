from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
HARNESS_ROOT = REPO_ROOT / ".codex" / "harness"
SCHEMA_ROOT = HARNESS_ROOT / "schemas"
SPEC_ROOT = HARNESS_ROOT / "evals" / "specs"
EXAMPLE_ROOT = HARNESS_ROOT / "evals" / "examples"
RUN_ROOT = HARNESS_ROOT / "evals" / "runs"
WORKSPACE_ROOT = HARNESS_ROOT / "evals" / "workspaces"
MATERIAL_ROOT = HARNESS_ROOT / "evals" / "materials"

STAGE_ORDER = [
    "INTAKE",
    "ADMISSION",
    "DISCOVERY",
    "RESEARCH",
    "DESIGN",
    "DESIGN_CRITIQUE",
    "WAIT_USER_APPROVAL",
    "IMPLEMENT",
    "VERIFY",
    "REVIEW",
    "DELIVER",
    "BLOCKED",
]

ARTIFACT_SCHEMA_MAP = {
    "manifest": "session_manifest.schema.json",
    "admission_result": "admission_result.schema.json",
    "intake_brief": "intake_brief.schema.json",
    "research_brief": "research_brief.schema.json",
    "design_packet": "design_packet.schema.json",
    "critique_report": "critique_report.schema.json",
    "gate_result": "gate_result.schema.json",
    "verification_report": "verification_report.schema.json",
    "delivery_report": "delivery_report.schema.json",
    "workflow_state": "workflow_state.schema.json",
    "metadata": "session_workspace.schema.json",
}

WORKSPACE_ARTIFACT_NAMES = {
    "admission_result",
    "intake_brief",
    "research_brief",
    "design_packet",
    "critique_report",
    "gate_result",
    "verification_report",
    "delivery_report",
    "workflow_state",
}

TRANSCRIPT_CANDIDATE_NAMES = (
    "transcript.md",
    "conversation.md",
    "chat.md",
    "session.md",
    "messages.md",
)

METADATA_CANDIDATE_NAMES = (
    "metadata.json",
    "task-metadata.json",
    "workspace.json",
)


def configure_utf8_stdio() -> None:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")


class MiniSchemaValidator:
    def __init__(self) -> None:
        self.errors: list[str] = []

    def validate(self, schema: dict[str, Any], data: Any, context: str = "$") -> list[str]:
        self.errors = []
        self._validate(schema, data, context)
        return self.errors

    def _validate(self, schema: dict[str, Any], data: Any, context: str) -> None:
        if "enum" in schema and data not in schema["enum"]:
            self.errors.append(f"{context}: value {data!r} not in enum {schema['enum']!r}")
            return

        schema_type = schema.get("type")
        if schema_type == "object":
            if not isinstance(data, dict):
                self.errors.append(f"{context}: expected object")
                return
            required = schema.get("required", [])
            for key in required:
                if key not in data:
                    self.errors.append(f"{context}: missing required key {key!r}")
            properties = schema.get("properties", {})
            if schema.get("additionalProperties") is False:
                unexpected = sorted(set(data.keys()) - set(properties.keys()))
                for key in unexpected:
                    self.errors.append(f"{context}: unexpected property {key!r}")
            for key, value in data.items():
                if key in properties:
                    self._validate(properties[key], value, f"{context}.{key}")
            return

        if schema_type == "array":
            if not isinstance(data, list):
                self.errors.append(f"{context}: expected array")
                return
            item_schema = schema.get("items")
            if item_schema:
                for index, item in enumerate(data):
                    self._validate(item_schema, item, f"{context}[{index}]")
            return

        if schema_type == "string":
            if not isinstance(data, str):
                self.errors.append(f"{context}: expected string")
            return

        if schema_type == "boolean":
            if not isinstance(data, bool):
                self.errors.append(f"{context}: expected boolean")
            return

        if schema_type == "integer":
            if isinstance(data, bool) or not isinstance(data, int):
                self.errors.append(f"{context}: expected integer")
            return

        if schema_type == "number":
            if isinstance(data, bool) or not isinstance(data, (int, float)):
                self.errors.append(f"{context}: expected number")
            return


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def bundle_artifact_path(bundle_root: Path, artifact_name: str) -> Path:
    if artifact_name == "manifest":
        return bundle_root / "manifest.json"
    if artifact_name == "transcript":
        return bundle_root / "transcript.md"
    return bundle_root / "artifacts" / f"{artifact_name}.json"


def schema_name_for_artifact(artifact_name: str) -> str | None:
    return ARTIFACT_SCHEMA_MAP.get(artifact_name)


def validate_with_named_schema(artifact_name: str, data: Any) -> list[str]:
    schema_name = schema_name_for_artifact(artifact_name)
    if not schema_name:
        return []
    validator = MiniSchemaValidator()
    return validator.validate(load_json(SCHEMA_ROOT / schema_name), data)


def infer_completed_stages(
    artifact_names: set[str],
    current_state: str | None,
    user_approval_granted: bool,
) -> list[str]:
    stages: set[str] = {"INTAKE"}

    if current_state in STAGE_ORDER and current_state != "BLOCKED":
        for stage in STAGE_ORDER:
            if stage == "BLOCKED":
                break
            stages.add(stage)
            if stage == current_state:
                break

    if "admission_result" in artifact_names:
        stages.add("ADMISSION")
    if "intake_brief" in artifact_names:
        stages.add("DISCOVERY")
    if "research_brief" in artifact_names:
        stages.add("RESEARCH")
    if "design_packet" in artifact_names:
        stages.add("DESIGN")
    if "critique_report" in artifact_names or "gate_result" in artifact_names:
        stages.add("DESIGN_CRITIQUE")
    if user_approval_granted or current_state in {"WAIT_USER_APPROVAL", "IMPLEMENT", "VERIFY", "REVIEW", "DELIVER"}:
        stages.add("WAIT_USER_APPROVAL")
    if current_state in {"IMPLEMENT", "VERIFY", "REVIEW", "DELIVER"} or "delivery_report" in artifact_names:
        stages.add("IMPLEMENT")
    if current_state in {"VERIFY", "REVIEW", "DELIVER"} or "verification_report" in artifact_names:
        stages.add("VERIFY")
    if current_state in {"REVIEW", "DELIVER"}:
        stages.add("REVIEW")
    if current_state == "DELIVER" or "delivery_report" in artifact_names:
        stages.add("DELIVER")
    if current_state == "BLOCKED":
        stages.add("BLOCKED")

    return [stage for stage in STAGE_ORDER if stage in stages]
