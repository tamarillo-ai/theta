---
name: security-review
description: Review code changes for security vulnerabilities, authentication gaps, injection risks, and unsafe patterns. Use before merging PRs or after security-sensitive changes.
---

## Diff to review

Review the current branch changes against main:
```bash
git diff main...HEAD
```

If no diff is available, review the most recent commit:
```bash
git diff HEAD~1
```

## Audit the changes for:

1. **Injection vulnerabilities** — SQL injection, XSS, command injection via unsanitized input
2. **Authentication & authorization gaps** — missing auth checks, broken RBAC, token handling issues
3. **Hardcoded secrets** — API keys, passwords, tokens in source code
4. **Path traversal** — unsanitized file paths that could escape intended directories
5. **Unsafe subprocess calls** — raw subprocess without CLICommunicator, shell=True usage
6. **Dependency issues** — known vulnerable packages, pinning concerns

## Use the checklist in this skill directory
See `references/checklist.md` for the full security review checklist.

## Report format
For each finding:
- **Severity**: Critical / High / Medium / Low
- **File**: path and line number
- **Issue**: what the vulnerability is
- **Fix**: concrete remediation steps
