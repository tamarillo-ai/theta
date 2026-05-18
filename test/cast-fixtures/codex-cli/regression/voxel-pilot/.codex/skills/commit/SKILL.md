---
name: "commit"
description: "Create a git commit with a properly formatted message following the project's commit convention. Reads staged changes and generates a subject line with the correct prefix (feat:, fix:, refactor:, test:, docs:, chore:, задача:). Use when you are ready to commit completed work."
metadata:
  short-description: "Commit staged changes with a correctly formatted message"
---

<objective>
Read the current git state, generate a commit message that matches the project's commit convention,
confirm with the user if needed, and create the commit.

Project convention (from AGENTS.md):
- Use short imperative subject lines
- Prefix with: feat:, fix:, refactor:, test:, docs:, chore:, задача:
- Example: "feat: add primitiveSmelt actor and HSM wiring"
- Example: "задача: wire anti-loop guard into AGENT_LOOP"
</objective>

<context>
Arguments: {{GSD_ARGS}}

Git state:
- Staged files: !`git diff --cached --name-only`
- Staged diff summary: !`git diff --cached --stat`
- Current branch: !`git branch --show-current`
- Last 3 commits (for style reference): !`git log --oneline -3`
</context>

<process>
1. **Assess staged changes**
   - If nothing is staged, check `git status` and inform the user — do not proceed with an empty commit.
   - If `{{GSD_ARGS}}` contains a commit message, use it directly (add prefix if missing).
   - Otherwise, derive the message from the staged diff.

2. **Choose the correct prefix**
   | Change type | Prefix |
   |-------------|--------|
   | New feature / capability | `feat:` |
   | Bug fix | `fix:` |
   | Refactor / cleanup, no behavior change | `refactor:` |
   | Tests only | `test:` |
   | Documentation only | `docs:` |
   | Build, config, tooling | `chore:` |
   | Task / milestone work (Russian context) | `задача:` |

3. **Draft the subject line**
   - ≤ 72 characters
   - Imperative mood ("add", "fix", "wire", not "added" or "adds")
   - No period at the end

4. **Add optional body** (only if the change is non-trivial)
   - Blank line after subject
   - Explain *what* changed and *why*, not *how*

5. **Execute the commit**
   ```bash
   git commit -m "<subject>" [-m "<body>"]
   ```

6. **Confirm** — report the full commit hash and subject line after success.
</process>
