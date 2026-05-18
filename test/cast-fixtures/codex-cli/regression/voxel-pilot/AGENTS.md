# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the runtime code. `src/index.ts` is the entrypoint. `src/core/` owns bot bootstrap, chat command ingestion, HSM wiring, and persistent services. `src/ai/` contains the LLM agent loop, provider clients, snapshot builder, tool schemas, and tool execution helpers. `src/core/memory/` is the long-term memory layer backed by SQLite via `better-sqlite3`. `src/hsm/` contains the XState v5 machine, primitives, actors, guards, and HSM utilities. Shared typings live in `src/types/`, config in `src/config/`, Mineflayer plugin wiring in `src/modules/`, and generic helpers in `src/utils/`. Tests live under `src/tests/` grouped by subsystem (`ai`, `core`, `config`, `hsm`). Build output goes to `dist/`, runtime state to `data/`, and logs to `logs/`.

## Architecture Notes
The task system is no longer a hardcoded `MINING/FARMING/CRAFTING` planner. The bot now runs an `AGENT_LOOP` inside `TASKS` with the shape `IDLE -> THINKING -> EXECUTING`. `THINKING` builds a deterministic snapshot and asks the configured model for one tool decision. Informational tools execute inline; execution tools transition the HSM into a concrete primitive state. `EXECUTING` writes success or failure back into machine context and always returns to `THINKING` unless the goal is cleared or the anti-loop guard aborts the run.

## Memory Layer
Do not add direct `fs` persistence for agent memory. Use the memory manager under `src/core/memory/` only. Persistent memory is stored in SQLite files under `data/` and accessed through CRUD methods such as `saveEntry`, `readEntries`, `updateEntryData`, `deleteEntry`, and container inspection helpers. If the schema changes, add a proper migration path instead of bolting new fields onto ad hoc JSON blobs.

## `.codex/agents`
Project-specific subagent definitions live in `.codex/agents/`. Current agents are:
- `hardening-worker.toml`
- `hsm-mapper.toml`
- `mineflayer-archivist.toml`
- `reviewer-hardline.toml`
- `test-writer.toml`

Use them when the task matches their ownership. Keep agent roles narrow, concrete, and non-overlapping. If you add a new subagent, document its purpose in the TOML clearly and keep it aligned with the actual repo architecture.

When subagents need to orient themselves in the codebase, prefer Serena MCP for symbol lookup, references, and targeted file discovery before falling back to plain text search such as `rg`. This is a recommendation, not a hard requirement: `rg` is still fine for broad text hunts or non-code files. The reason is simple: Serena usually gives more precise symbol-level navigation, reduces accidental grep-driven guesses, and helps agents touch fewer irrelevant files.

## Build, Test, and Development Commands
Use Node 18+; current project dependencies are already aligned with modern Node and ESM. Core commands:

- `npm run dev`: run the bot with `tsx watch src/index.ts`
- `npm run build`: compile TypeScript and rewrite path aliases with `tsc-alias`
- `npm start`: run the compiled bot from `dist/index.js`
- `npm run type-check`: strict TypeScript validation without emitting files
- `npm run knip`: detect unused files, exports, and dependencies
- `npm run clean`: remove `dist/`
- `npx tsx --test src/tests/...`: run focused subsystem tests

Before committing, run at least `npm run type-check` and `npm run build`. For HSM, AI, or memory changes, also run the relevant `tsx --test` suites under `src/tests/`.

## Coding Style & Naming Conventions
Formatting is defined by `.prettierrc`: tabs, width 2, no semicolons, single quotes, ES modules. Prefer path aliases such as `@/core/*`, `@/hsm/*`, `@/utils/*`, and `@/ai/*` over deep relative imports. Follow existing domain naming: classes in PascalCase, functions and variables in camelCase, and state-machine files with explicit suffixes such as `*.guards.ts`, `*.actors.ts`, `*.primitive.ts`, and `*.update.ts`.

## Testing Guidelines
Tests already exist in `src/tests/`; do not pretend the project is untested. Add focused tests next to the affected subsystem directory. Prioritize regression tests for HSM transitions, AI tool parsing, snapshot formatting, provider clients, and memory CRUD semantics. If a bug only reproduces in the live Minecraft session, document the manual scenario precisely in `docs/` or in the change summary.

## Commit & Pull Request Guidelines
Keep commit subjects short and imperative. Existing history uses prefixes like `feat:` and `задача:`; continue that style. A valid PR or handoff note should state which subsystem changed, what behavioral contract changed, and how it was verified. Do not dump raw terminal noise when a concise verification summary is enough.

## Security & Configuration Tips
Start from `.env.example` and keep secrets only in local `.env`. Never hardcode provider keys, server addresses, or tokens in source files, tests, or docs. Review `data/` and `logs/` before committing. If you rotate model providers, update `.env.example` and the config contract together; do not leave stale environment documentation behind.
