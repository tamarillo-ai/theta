---
name: generate-reports
description: Generate all orchestrator report types — execution summaries, agent performance, workflow analytics, health, config audit, and HTML dashboard with charts. Use after task runs or for project status overview.
---

Generate reports for the AI Coding Tools Orchestrator.

## Generate all reports
```bash
python3 -c "
import yaml
from orchestrator.observability.report_generator import ReportGenerator

with open('orchestrator/config/agents.yaml') as f:
    config = yaml.safe_load(f)

gen = ReportGenerator(reports_dir='./reports')
paths = gen.seed_reports(config=config)
for p in paths:
    print(f'  Generated: {p}')
print(f'\n{len(paths)} reports generated in reports/')
"
```

## Report types
- **perf_*.json** — Agent performance: success rates, call counts, task type distribution
- **workflow_*.json** — Workflow analytics: per-workflow runs, success rates, avg iterations
- **health_*.json** — System health: Python version, disk, memory, dependencies
- **config_*.json** — Config audit: agent availability, workflow structure, settings
- **dashboard_*.html** — Interactive HTML dashboard with Chart.js (4 charts + KPI cards)

## After generating
Report which files were created and their sizes. Mention the HTML dashboard can be opened in a browser.
