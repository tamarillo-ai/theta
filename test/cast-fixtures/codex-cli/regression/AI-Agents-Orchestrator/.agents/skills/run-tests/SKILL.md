---
name: run-tests
description: Run the project test suite with optional filtering by marker or file. Use when you need to verify code changes, check test status, or diagnose test failures.
---

Run the test suite for the AI Coding Tools Orchestrator.

## Default (unit tests only)
```bash
python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -m "not integration and not slow"
```

## By marker
If the user specifies a marker (unit, integration, slow, security, agentic_team):
```bash
python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -m "<marker>"
```

## By file
If the user specifies a test file:
```bash
python -m pytest tests/test_<name>.py --override-ini="addopts=" -q --timeout=30
```

## After running
1. Report: total passed, failed, errors
2. For each failure: test name, error message, likely root cause
3. If all pass, confirm with the count
