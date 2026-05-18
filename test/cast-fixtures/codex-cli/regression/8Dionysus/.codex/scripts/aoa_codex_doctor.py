
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


SCRIPT_DIR = Path(__file__).expanduser().resolve().parent
WORKSPACE_ROOT = SCRIPT_DIR.parents[1]
TOOLS_DIR = WORKSPACE_ROOT / ".codex" / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from aoa_codex_converge.convergence_state import build_report, report_to_markdown


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Doctor for AoA Codex convergence state.")
    parser.add_argument("--workspace-root", default=".", help="AoA workspace root.")
    parser.add_argument("--write-report", action="store_true", help="Write JSON and Markdown reports.")
    parser.add_argument(
        "--report-dir",
        default=None,
        help="Directory for generated reports. Defaults to <workspace-root>/.codex/generated/codex.",
    )
    parser.add_argument("--format", choices=["text", "json", "markdown"], default="text")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    workspace_root = Path(args.workspace_root).expanduser().resolve()
    report = build_report(workspace_root)

    if args.write_report:
        report_dir = (
            Path(args.report_dir).expanduser().resolve()
            if args.report_dir
            else workspace_root / ".codex" / "generated" / "codex"
        )
        report_dir.mkdir(parents=True, exist_ok=True)
        (report_dir / "aoa_codex_convergence_report.json").write_text(
            json.dumps(report, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        (report_dir / "aoa_codex_convergence_report.md").write_text(
            report_to_markdown(report),
            encoding="utf-8",
        )

    if args.format == "json":
        print(json.dumps(report, indent=2, ensure_ascii=False))
    elif args.format == "markdown":
        print(report_to_markdown(report))
    else:
        print(f"ready={report['ready']}")
        for surface in report["surfaces"]:
            print(f"{surface['status']:>7}  {surface['name']}  {surface['summary']}")

    return 0 if report["ready"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
