# Report Types Reference

## JSON Reports

| Type | File Pattern | Contents |
|---|---|---|
| Execution Summary | `exec_*.json` | Per-task steps, agents, fallbacks, suggestions, duration |
| Agent Performance | `perf_*.json` | Aggregated success rates, call counts, task type distribution |
| Workflow Analytics | `workflow_*.json` | Per-workflow runs, success rates, average iterations |
| System Health | `health_*.json` | Health checks, disk/memory, Python version, platform |
| Config Audit | `config_*.json` | Agent availability, workflow structure, settings snapshot |

## HTML Dashboard

| File Pattern | Charts |
|---|---|
| `dashboard_*.html` | Daily task volume (bar), agent success/failure (stacked bar), duration trend (line), workflow distribution (doughnut) |

## Index

`INDEX.json` — catalog of all generated reports with timestamps.

## Programmatic Usage

```python
from orchestrator.observability import ReportGenerator

gen = ReportGenerator(reports_dir="./reports")

# Single report
gen.generate_health_report()
gen.generate_config_audit(config)
gen.generate_execution_report(task, workflow, results, duration, agents)

# All reports with sample data
gen.seed_reports(config=config)

# HTML dashboard
gen.generate_html_dashboard()
```
