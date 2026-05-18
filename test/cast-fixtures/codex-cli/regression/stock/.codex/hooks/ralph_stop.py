from __future__ import annotations

import json
import os
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONTROL_FILE = ROOT / ".omx" / "state" / "RALPH_CONTROL_STATE.json"
RUNTIME_STATUS_FILE = ROOT / ".omx" / "runtime" / "omx-loop-status.json"
STATE_FILE = ROOT / ".ralph" / "state.json"
PROGRESS_FILE = ROOT / ".ralph" / "progress.md"


def load_json(path: Path, default: dict) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return default
    return payload if isinstance(payload, dict) else default


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    payload = json.load(sys.stdin) if not sys.stdin.closed else {}
    state = load_json(STATE_FILE, {"enabled": True, "iteration": 0})
    control = load_json(CONTROL_FILE, {"command": "", "nudge": "", "goal_override": "", "done_overrides": []})
    runtime = load_json(RUNTIME_STATUS_FILE, {})

    completion_token = os.getenv("RALPH_COMPLETION_TOKEN", "RALPH_DONE")
    block_token = os.getenv("RALPH_BLOCK_TOKEN", "RALPH_BLOCKED")
    max_iterations = int(os.getenv("RALPH_MAX_ITERATIONS", "30"))
    last_message = str(payload.get("last_assistant_message", "") or "")

    if completion_token in last_message or block_token in last_message:
        state["enabled"] = False
        write_json(STATE_FILE, state)
        print(json.dumps({"continue": True}))
        return 0

    command = str(control.get("command", "") or "").strip().lower()
    if command == "stop":
        print(json.dumps({"continue": False, "stopReason": "Discord requested stop"}))
        return 0
    if command == "pause":
        print(json.dumps({"continue": False, "stopReason": "Discord requested pause"}))
        return 0

    state["iteration"] = int(state.get("iteration", 0)) + 1
    write_json(STATE_FILE, state)
    if state["iteration"] >= max_iterations:
        print(json.dumps({"continue": False, "stopReason": f"Max iterations reached: {max_iterations}"}))
        return 0

    nudge = str(control.get("nudge", "") or "").strip()
    goal_override = str(control.get("goal_override", "") or "").strip()
    done_overrides = control.get("done_overrides", [])
    if not isinstance(done_overrides, list):
        done_overrides = []

    lines = [
        "Continue the Ralph loop.",
        f"Iteration: {state['iteration']}/{max_iterations}",
        f"Runtime status: {runtime.get('status', 'unknown')}",
    ]
    if PROGRESS_FILE.exists():
        lines.append("Read .ralph/progress.md before continuing.")
    if goal_override:
        lines.append(f"Updated goal override: {goal_override}")
    if done_overrides:
        lines.append("Additional done conditions:")
        lines.extend(f"- {item}" for item in done_overrides if str(item).strip())
    if nudge:
        lines.append(f"Discord nudge: {nudge}")
    lines.extend(
        [
            f"If complete, output {completion_token}.",
            f"If blocked on payment, credentials, external accounts, deployment, or irreversible operations, output {block_token}.",
        ]
    )
    print(json.dumps({"decision": "block", "reason": "\n".join(lines)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
