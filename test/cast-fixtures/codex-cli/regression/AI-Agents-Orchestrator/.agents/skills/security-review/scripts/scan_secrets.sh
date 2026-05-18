#!/usr/bin/env bash
# Scan for hardcoded secrets in the codebase.
# Usage: ./scan_secrets.sh [path]

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

TARGET="${1:-.}"

echo "=== Scanning for hardcoded secrets in $TARGET ==="
echo ""

# Common secret patterns
PATTERNS=(
    "password\s*=\s*['\"][^'\"]+['\"]"
    "secret\s*=\s*['\"][^'\"]+['\"]"
    "api_key\s*=\s*['\"][^'\"]+['\"]"
    "token\s*=\s*['\"][^'\"]+['\"]"
    "AWS_ACCESS_KEY"
    "PRIVATE.KEY"
    "BEGIN RSA PRIVATE KEY"
    "BEGIN OPENSSH PRIVATE KEY"
)

FOUND=0
for pattern in "${PATTERNS[@]}"; do
    RESULTS=$(grep -rn --include="*.py" --include="*.yaml" --include="*.yml" --include="*.json" --include="*.toml" -iE "$pattern" "$TARGET" 2>/dev/null | grep -v "node_modules" | grep -v ".git/" | grep -v "venv/" || true)
    if [[ -n "$RESULTS" ]]; then
        echo "Pattern: $pattern"
        echo "$RESULTS" | head -5
        echo ""
        FOUND=$((FOUND + 1))
    fi
done

if [[ $FOUND -eq 0 ]]; then
    echo "✓ No hardcoded secrets found."
else
    echo "⚠ Found $FOUND pattern categories with potential secrets."
    echo "Review each finding — some may be test fixtures or placeholders."
fi
