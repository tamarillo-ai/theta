#!/usr/bin/env python3
"""Run health checks and generate a report."""

import json
import sys

sys.path.insert(0, ".")

from orchestrator.observability.health import HealthChecker
from orchestrator.observability.report_generator import ReportGenerator


def main():
    checker = HealthChecker()
    results = checker.run_all_checks()

    print("=" * 60)
    print(f"Overall Status: {results['status'].upper()}")
    print(f"Duration: {results['duration_ms']:.1f}ms")
    print("=" * 60)

    for check in results["checks"]:
        icon = (
            "✓" if check["status"] == "healthy" else "⚠" if check["status"] == "degraded" else "✗"
        )
        print(f"  {icon} {check['name']}: {check['message']}")

    print()
    gen = ReportGenerator(reports_dir="./reports")
    path = gen.generate_health_report(health_results=results)
    print(f"Report saved: {path}")


if __name__ == "__main__":
    main()
