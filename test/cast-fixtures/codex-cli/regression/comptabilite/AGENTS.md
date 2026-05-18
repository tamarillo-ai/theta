# AGENTS.md

## Purpose

This repository is a local-first accounting system for a solo incorporated IT consultant in Quebec.

Primary goal:
- keep the Beancount ledger correct, auditable, and CPA-friendly

Non-goals:
- do not present tax/legal opinions as facts
- do not fabricate accounting categories, journal entries, or tax amounts
- do not build filing software for T2, CO-17, T1, TP-1, T4, RL-1, or GST/QST returns

When uncertain, prefer surfacing ambiguity over automating through it.

## Business Context

- Jurisdiction: Quebec, Canada
- Entity: CCPC
- Domain constraints: GST 5%, QST 9.975%, Quebec payroll contributions, CCA/DPA classes, shareholder loan rules
- Human CPA remains the final authority for tax filing and planning

This matters because code changes should optimize for traceability, deterministic formulas, and clean export packages for a CPA review workflow.

## Operating Model

The repo is centered on a plain-text Beancount ledger. Python code layers import, classification, Quebec-specific calculations, document handling, reports, MCP tools, and Fava extensions around that ledger.

Source of truth:
- `ledger/main.beancount`
- included monthly ledger files under `ledger/`
- chart of accounts in `ledger/comptes.beancount`
- approval queue in `ledger/pending.beancount`

High-level surfaces:
- CLI: `src/compteqc/cli/app.py`
- MCP server: `src/compteqc/mcp/server.py`
- Fava UI: `ledger/main.beancount` plus `src/compteqc/fava_ext/`
- reports/export: `src/compteqc/rapports/`

## Repo Map

Core code:
- `src/compteqc/ingestion/`: bank and card importers, normalization
- `src/compteqc/categorisation/`: rules, ML, LLM, approval queue logic
- `src/compteqc/ledger/`: Beancount file writes, validation, git helpers
- `src/compteqc/quebec/`: payroll, taxes, DPA, shareholder loan logic
- `src/compteqc/documents/`: receipt and invoice extraction, matching, Beancount document links
- `src/compteqc/factures/`: accounts receivable and invoice generation
- `src/compteqc/fournisseurs/`: accounts payable
- `src/compteqc/rapports/`: CPA package, statements, summaries
- `src/compteqc/mcp/`: FastMCP server and tool modules
- `src/compteqc/fava_ext/`: Fava tabs and extension UI

Project data and config:
- `rules/`: editable categorization and tax rules
- `data/`: file-backed registries, processed imports, ML artifacts, corrections
- `docs/`: design docs
- `.planning/`: GSD planning artifacts and codebase maps

Tests:
- `tests/`: pytest suite by subsystem
- `tests/fixtures/`: real-format sample input files

## How To Work In This Repo

### 1. Start from the right boundary

If the task is about:
- import formats or transaction normalization: start in `src/compteqc/ingestion/`
- category proposals or approval queue behavior: start in `src/compteqc/categorisation/`
- journal posting and Beancount writes: start in `src/compteqc/ledger/`
- payroll, GST/QST, DPA, shareholder loan formulas: start in `src/compteqc/quebec/`
- accountant-facing outputs: start in `src/compteqc/rapports/`
- operator commands: start in `src/compteqc/cli/`
- AI tool access: start in `src/compteqc/mcp/`
- review UI: start in `src/compteqc/fava_ext/`

### 2. Preserve local-first assumptions

- Prefer file-backed state over adding a database.
- Keep business logic out of templates and UI handlers.
- Treat ledger mutations as sensitive operations that must validate cleanly.
- Use absolute, reproducible inputs where possible.

### 3. Match the codebase style

- Most domain modules and tests use French identifiers and docstrings.
- Keep public behavior explicit and deterministic.
- Prefer pure functions for formulas and summaries.
- Reuse existing patterns before introducing new abstractions.

## Important Entry Points

CLI commands:
- `cqc` console script from `pyproject.toml`
- app root: `src/compteqc/cli/app.py`

MCP server:
- module entry: `src/compteqc/mcp/__main__.py`
- server definition: `src/compteqc/mcp/server.py`
- tool groups:
  - `src/compteqc/mcp/tools/ledger.py`
  - `src/compteqc/mcp/tools/quebec.py`
  - `src/compteqc/mcp/tools/categorisation.py`
  - `src/compteqc/mcp/tools/approbation.py`
  - `src/compteqc/mcp/tools/paie.py`
  - `src/compteqc/mcp/tools/apar.py`

Fava:
- Beancount launch file: `ledger/main.beancount`
- extensions are registered from the ledger via custom directives

## Safe Change Rules

- Do not invent ledger entries or tax results to make tests pass.
- Do not silently change account naming or posting behavior without tracing downstream report impact.
- Do not weaken validation around pending approval flows.
- Be careful with any code that rewrites `ledger/pending.beancount` or monthly ledger files.
- Changes in `src/compteqc/quebec/` should usually come with focused formula tests.
- Changes in `src/compteqc/mcp/` should preserve read-only behavior when `COMPTEQC_READONLY=true`.

## Validation Commands

Environment and tests:

```bash
uv run pytest
uv run pytest tests/test_mcp_server.py
uv run pytest tests/test_mcp_mutations.py
uv run pytest tests/test_importers.py
uv run pytest tests/test_rates.py
uv run pytest --cov=compteqc
uv run ruff check .
uv run mypy src
```

CLI and ledger workflows:

```bash
uv run cqc --help
uv run python -m compteqc.mcp
uv run fava ledger/main.beancount
```

Use narrower test slices whenever possible for fast iteration.

## MCP Setup

This repo ships its own local MCP server, `compteqc`, for ledger queries and mutations.

Claude project configuration currently uses:

```json
{
  "compteqc": {
    "type": "stdio",
    "command": "uv",
    "args": ["run", "python", "-m", "compteqc.mcp"]
  }
}
```

For Codex, the equivalent setup should launch the same server from this project directory so the agent can query the live ledger and use the same tool surface.

Recommended Codex registration:

```bash
codex mcp add compteqc -- \
  uv run --directory /Users/philippebeliveau/Desktop/Notebook/comptabilite python -m compteqc.mcp
```

Useful environment variables:
- `COMPTEQC_LEDGER=/Users/philippebeliveau/Desktop/Notebook/comptabilite/ledger/main.beancount`
- `COMPTEQC_READONLY=true` for query-only sessions

## Current Architecture Notes

- The repo is already committed to Beancount, not PyLedger.
- Fava is the main interactive UI layer.
- MCP is the automation and assistant-control layer.
- AP/AR work exists and should be treated as active product surface, not a throwaway experiment.
- `.planning/codebase/` contains current architecture, structure, conventions, testing, integrations, and concern maps.

## Current Working Tax Assumptions

These are working assumptions for analysis and draft bookkeeping only. They are not tax facts and remain subject to CPA review.

- Home-office rent allocation: use `20%` of apartment rent as the current working estimate.
- Home internet allocation: use `50%` of home internet charges as the current working estimate.
- Software used exclusively for work may be treated as `100%` business-use pending receipt/support review.
- For the current personal March 2026 review, the working software items are:
  - `Google Cloud 42.75`
  - `Google One 15.51`
  - `Microsoft 2.30`
- Any home-office claim discussion should cite current CRA and Revenu Quebec guidance and should not present these proportions as legally confirmed entitlements.

## Preferred Agent Behavior

- Read the relevant subsystem before proposing architecture changes.
- Keep explanations concrete and grounded in file paths.
- When discussing tradeoffs, separate bookkeeping correctness from UX convenience.
- For financial logic, prefer evidence from code, tests, and official rule sources over assumptions.
- If a task touches formulas or posting rules, explain exactly what changed and how it was verified.
