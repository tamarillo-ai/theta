#!/usr/bin/env python3
"""Generate all report types for the orchestrator."""

import sys

sys.path.insert(0, ".")

import yaml

from orchestrator.observability.report_generator import ReportGenerator


def main():
    with open("orchestrator/config/agents.yaml") as f:
        config = yaml.safe_load(f)

    gen = ReportGenerator(reports_dir="./reports")
    paths = gen.seed_reports(config=config)

    print("=" * 60)
    print("Report Generation Complete")
    print("=" * 60)
    for p in paths:
        size = p.stat().st_size
        print(f"  {p.name:40s} {size:>6,} bytes")
    print(f"\n{len(paths)} reports generated in reports/")
    print("Open dashboard_*.html in a browser for the interactive view.")


if __name__ == "__main__":
    main()
