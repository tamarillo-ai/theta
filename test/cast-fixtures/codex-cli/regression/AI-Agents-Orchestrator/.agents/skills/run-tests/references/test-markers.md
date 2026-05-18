# Test Markers Reference

| Marker | Description | CI Status |
|---|---|---|
| `unit` | Fast unit tests, no external dependencies | Runs in CI |
| `integration` | Requires CLI tools (claude, codex, gemini) | Skipped in CI |
| `slow` | Long-running tests (>30s) | Skipped in CI |
| `security` | Security-focused tests (bandit, injection) | Runs in CI |
| `agentic_team` | Agentic team runtime tests | Runs in CI |

## Common Commands

```bash
# Unit tests only (default for CI)
python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -m "not integration and not slow"

# All tests
python -m pytest tests/ --override-ini="addopts=" -q --timeout=30

# Single file
python -m pytest tests/test_orchestrator.py -q --override-ini="addopts=" --timeout=30

# By keyword
python -m pytest tests/ -q --override-ini="addopts=" --timeout=30 -k "adapter"
```

## Test File Map

| File | Covers |
|---|---|
| `test_orchestrator.py` | Core engine, workflow execution |
| `test_adapters.py` | All adapter implementations |
| `test_adapter_execution.py` | Adapter execution paths |
| `test_agentic_team_engine.py` | Agentic team runtime |
| `test_report_generator.py` | Report generation module |
| `test_production_hardening.py` | Thread safety, resource management |
| `test_enterprise_hardening.py` | Enterprise features |
| `test_security.py` | Security module |
| `test_functional_e2e.py` | End-to-end (integration) |
| `test_integration.py` | CLI communication (integration) |
