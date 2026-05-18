---
name: ship-checker
description: 'Run a full pre-ship review for this repo. Use when asked if a change is ready to deploy, publish, or ship.'
argument-hint: 'Describe scope (single route or full site) and whether fixes should be applied automatically.'
---

# Ship Checker

Use this skill when the user asks any form of:
- "Is this ready to ship?"
- "Can you review before deploy?"
- "Do a release readiness check"

## Scope

- Entire repository by default, unless the user explicitly requests a narrower scope.
- Common targets:
  - `src/routes/**`
  - `src/lib/**`
  - `static/**`
  - `.github/**` for workflow/instructions changes

## Procedure

1. Determine review target.
- If there are uncommitted changes, review those.
- If clean, review latest commit(s) relevant to the request.

2. Collect changed files and metadata.
- `git status --short`
- `git log --oneline -n 5`
- `git show --name-status --pretty=oneline HEAD`

3. Review implementation quality.
- Check route wiring, link integrity, and referenced static assets.
- Check SEO and structured data on changed pages.
- Check accessibility basics including reduced-motion behavior.

4. Run required validation commands.
- `npm run check`
- `npm run build`

5. Produce a release verdict.
- `Ready` only when no High findings remain and validation gates pass.
- `Not Ready` otherwise, with precise blockers.

## Guardrails

- Do not modify generated artifacts under `build/`.
- Keep findings evidence-based with exact file references.
- If fixes are requested, apply the smallest safe patch set and re-run validation.

## Done Definition

- Findings are listed by severity with clear evidence.
- Required validation command outcomes are included.
- A binary ship verdict is provided.
