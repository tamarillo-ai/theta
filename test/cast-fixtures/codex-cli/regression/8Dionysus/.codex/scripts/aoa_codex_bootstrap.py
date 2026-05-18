
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

from aoa_codex_converge.convergence_state import build_report, report_to_markdown, write_bootstrap_scaffold


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Create safe AoA Codex convergence scaffold.")
    parser.add_argument("--workspace-root", default=".", help="AoA workspace root.")
    parser.add_argument(
        "--report-dir",
        default=None,
        help="Directory for generated reports. Defaults to <workspace-root>/.codex/generated/codex.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    workspace_root = Path(args.workspace_root).expanduser().resolve()
    report_dir = (
        Path(args.report_dir).expanduser().resolve()
        if args.report_dir
        else workspace_root / ".codex" / "generated" / "codex"
    )

    write_bootstrap_scaffold(workspace_root)
    report = build_report(workspace_root)

    report_dir.mkdir(parents=True, exist_ok=True)
    json_path = report_dir / "aoa_codex_convergence_report.json"
    md_path = report_dir / "aoa_codex_convergence_report.md"

    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    md_path.write_text(report_to_markdown(report), encoding="utf-8")

    print(f"Wrote {json_path}")
    print(f"Wrote {md_path}")
    print(f"ready={report['ready']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
