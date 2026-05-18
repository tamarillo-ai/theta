
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
    parser = argparse.ArgumentParser(description="Render AoA Codex convergence status.")
    parser.add_argument("--workspace-root", default=".", help="AoA workspace root.")
    parser.add_argument("--format", choices=["json", "markdown"], default="markdown")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    workspace_root = Path(args.workspace_root).expanduser().resolve()
    report = build_report(workspace_root)
    if args.format == "json":
        print(json.dumps(report, indent=2, ensure_ascii=False))
    else:
        print(report_to_markdown(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
