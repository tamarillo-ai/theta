#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).expanduser().resolve().parents[2]
TOOLS_DIR = WORKSPACE_ROOT / ".codex" / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from aoa_codex_hooks.events import handle_stop, parse_event


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="AoA Codex Stop hook doctor.")
    parser.add_argument(
        "--autocontinue-critical",
        action="store_true",
        help="Use Stop-hook continuation when critical seam errors are present.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    event = parse_event(sys.stdin.read())
    payload = handle_stop(event, WORKSPACE_ROOT, autocontinue_critical=args.autocontinue_critical)
    print(json.dumps(payload, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
