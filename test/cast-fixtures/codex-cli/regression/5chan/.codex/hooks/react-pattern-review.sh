#!/bin/bash

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
exec "$repo_root/scripts/agent-hooks/react-pattern-review.sh" --skill-dir .codex/skills --scope-prefix src/ "$@"
