# Health Checks Reference

## Checks Performed

| Check | What it verifies | Healthy | Degraded | Unhealthy |
|---|---|---|---|---|
| `python_version` | Python >= 3.8 | 3.8+ | — | < 3.8 |
| `disk_space` | Free disk space | > 5 GB | 1-5 GB | < 1 GB |
| `memory` | Available RAM | < 80% used | 80-90% used | > 90% used |
| `config_file` | agents.yaml valid | Present + valid | Missing sections | Not found |
| `directories` | Required dirs exist | All present | Some missing | — |
| `dependencies` | Python packages | All installed | — | Missing packages |

## Required Directories

- `output/` — generated code artifacts
- `workspace/` — session working directories
- `reports/` — JSON reports + HTML dashboard
- `sessions/` — saved session state
- `logs/` — application logs

## Required Dependencies

- `click` — CLI framework
- `pyyaml` — YAML configuration
- `rich` — Terminal output
- `pydantic` — Data validation

## Health Endpoints

- Orchestrator UI: `http://localhost:5001/health`, `/ready`
- Agentic Team UI: `http://localhost:5002/health`, `/ready`
