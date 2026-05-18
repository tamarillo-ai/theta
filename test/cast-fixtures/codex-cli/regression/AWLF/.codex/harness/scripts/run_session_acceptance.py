from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from harness_utils import EXAMPLE_ROOT, SCHEMA_ROOT, SPEC_ROOT, MiniSchemaValidator, bundle_artifact_path, configure_utf8_stdio, load_json


configure_utf8_stdio()


@dataclass
class CheckResult:
    passed: bool
    message: str

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run OM harness session acceptance checks.")
    parser.add_argument("--bundle", type=Path, help="Path to one session bundle directory.")
    parser.add_argument("--spec", type=Path, help="Path to one eval spec JSON file.")
    parser.add_argument(
        "--all-examples",
        action="store_true",
        help="Run all example bundles in .codex/harness/evals/examples.",
    )
    return parser.parse_args()


def load_spec(spec_path: Path) -> dict[str, Any]:
    return json.loads(spec_path.read_text(encoding="utf-8"))


def load_artifact(bundle_root: Path, artifact_name: str) -> Any:
    artifact_path = bundle_artifact_path(bundle_root, artifact_name)
    if artifact_name == "transcript":
        return artifact_path.read_text(encoding="utf-8")
    return load_json(artifact_path)


def resolve_path(data: Any, dotted_path: str) -> Any:
    current = data
    if dotted_path == "":
        return current
    for segment in dotted_path.split("."):
        while "[" in segment and "]" in segment:
            field_name = segment[: segment.index("[")]
            if field_name:
                current = current[field_name]
            index_text = segment[segment.index("[") + 1 : segment.index("]")]
            current = current[int(index_text)]
            segment = segment[segment.index("]") + 1 :]
        if segment:
            current = current[segment]
    return current


def run_check(bundle_root: Path, check: dict[str, Any], cache: dict[str, Any]) -> CheckResult:
    kind = check["kind"]

    if kind == "schema_valid":
        artifact = check["artifact"]
        schema_name = check["schema"]
        artifact_data = cache.setdefault(artifact, load_artifact(bundle_root, artifact))
        schema_data = load_json(SCHEMA_ROOT / schema_name)
        validator = MiniSchemaValidator()
        errors = validator.validate(schema_data, artifact_data)
        if errors:
            return CheckResult(False, f"{artifact}: schema validation failed: {'; '.join(errors)}")
        return CheckResult(True, f"{artifact}: schema valid")

    if kind == "field_equals":
        artifact = check["artifact"]
        value = resolve_path(cache.setdefault(artifact, load_artifact(bundle_root, artifact)), check["path"])
        expected = check["expected"]
        if value != expected:
            return CheckResult(False, f"{artifact}.{check['path']}: expected {expected!r}, got {value!r}")
        return CheckResult(True, f"{artifact}.{check['path']}: matched expected value")

    if kind == "bool_true":
        artifact = check["artifact"]
        value = resolve_path(cache.setdefault(artifact, load_artifact(bundle_root, artifact)), check["path"])
        if value is not True:
            return CheckResult(False, f"{artifact}.{check['path']}: expected true, got {value!r}")
        return CheckResult(True, f"{artifact}.{check['path']}: true")

    if kind == "array_min_length":
        artifact = check["artifact"]
        value = resolve_path(cache.setdefault(artifact, load_artifact(bundle_root, artifact)), check["path"])
        minimum = check["min"]
        if not isinstance(value, list) or len(value) < minimum:
            actual = len(value) if isinstance(value, list) else "non-array"
            return CheckResult(False, f"{artifact}.{check['path']}: expected array length >= {minimum}, got {actual}")
        return CheckResult(True, f"{artifact}.{check['path']}: length >= {minimum}")

    if kind == "transcript_contains_any":
        transcript = cache.setdefault("transcript", load_artifact(bundle_root, "transcript"))
        needles = check["needles"]
        if any(needle in transcript for needle in needles):
            return CheckResult(True, "transcript contains at least one expected phrase")
        return CheckResult(False, f"transcript missing any of {needles!r}")

    if kind == "transcript_contains_all":
        transcript = cache.setdefault("transcript", load_artifact(bundle_root, "transcript"))
        needles = check["needles"]
        missing = [needle for needle in needles if needle not in transcript]
        if missing:
            return CheckResult(False, f"transcript missing required phrases {missing!r}")
        return CheckResult(True, "transcript contains all required phrases")

    if kind == "stage_contains":
        manifest = cache.setdefault("manifest", load_artifact(bundle_root, "manifest"))
        stages = manifest["completed_stages"]
        expected = check["stage"]
        if expected not in stages:
            return CheckResult(False, f"manifest.completed_stages missing {expected!r}")
        return CheckResult(True, f"manifest.completed_stages contains {expected!r}")

    if kind == "stage_order":
        manifest = cache.setdefault("manifest", load_artifact(bundle_root, "manifest"))
        stages = manifest["completed_stages"]
        first = check["first"]
        then = check["then"]
        try:
            first_index = stages.index(first)
            then_index = stages.index(then)
        except ValueError:
            return CheckResult(False, f"manifest.completed_stages missing {first!r} or {then!r}")
        if first_index >= then_index:
            return CheckResult(False, f"manifest.completed_stages order invalid: {first!r} must come before {then!r}")
        return CheckResult(True, f"manifest.completed_stages order valid: {first!r} before {then!r}")

    if kind == "file_exists":
        target = bundle_artifact_path(bundle_root, check["artifact"])
        if not target.exists():
            return CheckResult(False, f"missing file {target}")
        return CheckResult(True, f"found file {target.name}")

    return CheckResult(False, f"unsupported check kind {kind!r}")


def run_spec(bundle_root: Path, spec_path: Path) -> int:
    spec = load_spec(spec_path)
    cache: dict[str, Any] = {}
    failures = 0
    print(f"\n== {spec['id']} | {spec['title']} ==")

    for artifact in spec.get("required_artifacts", []):
        artifact_path = bundle_artifact_path(bundle_root, artifact)
        if artifact_path.exists():
            print(f"[PASS] required artifact present: {artifact_path.name}")
        else:
            print(f"[FAIL] required artifact missing: {artifact_path}")
            failures += 1

    for check in spec.get("checks", []):
        result = run_check(bundle_root, check, cache)
        prefix = "[PASS]" if result.passed else "[FAIL]"
        print(f"{prefix} {result.message}")
        if not result.passed:
            failures += 1

    if failures == 0:
        print("Result: PASS")
    else:
        print(f"Result: FAIL ({failures} issue(s))")
    return failures


def run_all_examples() -> int:
    total_failures = 0
    for bundle_root in sorted(path for path in EXAMPLE_ROOT.iterdir() if path.is_dir()):
        manifest = json.loads((bundle_root / "manifest.json").read_text(encoding="utf-8"))
        spec_path = SPEC_ROOT / f"{manifest['eval_id']}.json"
        total_failures += run_spec(bundle_root, spec_path)
    return total_failures


def main() -> int:
    args = parse_args()
    if args.all_examples:
        return 1 if run_all_examples() else 0

    if not args.bundle or not args.spec:
        print("Either use --all-examples or provide both --bundle and --spec.", file=sys.stderr)
        return 2

    return 1 if run_spec(args.bundle, args.spec) else 0


if __name__ == "__main__":
    raise SystemExit(main())
