#!/usr/bin/env bash
# Run the project test suite with optional marker/file argument.
# Usage: ./run_tests.sh [marker-or-file]

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

ARG="${1:-}"

if [[ -z "$ARG" ]]; then
    echo "Running unit tests (excluding integration and slow)..."
    python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -m "not integration and not slow"
elif [[ "$ARG" == *.py ]]; then
    echo "Running specific test file: $ARG"
    python -m pytest "tests/$ARG" --override-ini="addopts=" -q --timeout=30
elif [[ "$ARG" =~ ^(unit|integration|slow|security|agentic_team)$ ]]; then
    echo "Running tests with marker: $ARG"
    python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -m "$ARG"
else
    echo "Running all tests matching: $ARG"
    python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -k "$ARG"
fi
