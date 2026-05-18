#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).expanduser().resolve().parents[2]
TOOLS_DIR = WORKSPACE_ROOT / ".codex" / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from aoa_codex_hooks.events import handle_user_prompt_submit, parse_event


def main() -> int:
    event = parse_event(sys.stdin.read())
    payload = handle_user_prompt_submit(event, WORKSPACE_ROOT)
    if payload:
        print(json.dumps(payload, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
