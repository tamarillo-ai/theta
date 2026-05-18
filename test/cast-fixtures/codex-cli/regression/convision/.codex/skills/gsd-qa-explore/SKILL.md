---
name: "gsd-qa-explore"
description: "Exploración QA funcional con navegador (Convision) — hallazgos estructurados para agente corrector"
metadata:
  short-description: "Browser QA exploration with structured FINDINGS output"
---

Siempre reinicia el golang antes de empezar las validaciones.

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-qa-explore`.
- Treat all user text after `$gsd-qa-explore` as `{{GSD_ARGS}}`.
- If no arguments are present, treat `{{GSD_ARGS}}` as empty.

## B. AskUserQuestion → request_user_input Mapping
GSD workflows use `AskUserQuestion` (Claude Code syntax). Translate to Codex `request_user_input`:

Parameter mapping:
- `header` → `header`
- `question` → `question`
- Options formatted as `"Label" — description` → `{label: "Label", description: "description"}`
- Generate `id` from header: lowercase, replace spaces with underscores

Batched calls:
- `AskUserQuestion([q1, q2])` → single `request_user_input` with multiple entries in `questions[]`

Multi-select workaround:
- Codex has no `multiSelect`. Use sequential single-selects, or present a numbered freeform list asking the user to enter comma-separated numbers.

Execute mode fallback:
- When `request_user_input` is rejected (Execute mode), present a plain-text numbered list and pick a reasonable default.

## C. Task() → spawn_agent Mapping
GSD workflows use `Task(...)` (Claude Code syntax). Translate to Codex collaboration tools:

Direct mapping:
- `Task(subagent_type="X", prompt="Y")` → `spawn_agent(agent_type="X", message="Y")`
- `Task(model="...")` → omit (Codex uses per-role config, not inline model selection)
- `fork_context: false` by default — GSD agents load their own context via `<files_to_read>` blocks

Parallel fan-out:
- Spawn multiple agents → collect agent IDs → `wait(ids)` for all to complete

Result parsing:
- Look for structured markers in agent output: `CHECKPOINT`, `PLAN COMPLETE`, `SUMMARY`, etc.
- `close_agent(id)` after collecting results from each agent
</codex_skill_adapter>

<objective>
Ejecutar exploración QA **en navegador** sobre Convision (login por rol, sidebar + rutas del mapa), documentar fallos/gaps con evidencia y producir un archivo **FINDINGS** listo para el agente corrector (regla `convision-qa-fixer`).

No sustituye `/gsd-verify-work`: ese flujo es UAT conversacional por fase; este es smoke/exploración autónoma con MCP de navegador.
</objective>

<test_users>
Contraseña común (seed local): **`password`**. Canónico: `docs/CREDENCIALES_PRUEBA_ROLES.md`.

**Genéricos (`UsersTableSeeder`):** `admin@convision.com`, `specialist@convision.com`, `receptionist@convision.com`.

**Demo (`DemoStaffSeeder`):** `cvargas@convision.com` (admin); `abermudez@convision.com`, `storres@convision.com`, `dmontoya@convision.com` (specialist); `vcastillo@convision.com`, `jnieto@convision.com` (receptionist); `hquintero@convision.com` (laboratory — el front puede mandar a `/unauthorized`).

**API JWT:** `POST http://localhost:8000/api/v1/auth/login` body `{"email":"…","password":"password"}`.
</test_users>

<execution_context>
@/Users/wilderherrera/Desktop/convision/.codex/get-shit-done/workflows/qa-explore.md
@/Users/wilderherrera/Desktop/convision/docs/QA_MAPA_EXPLORACION.md
@/Users/wilderherrera/Desktop/convision/docs/CREDENCIALES_PRUEBA_ROLES.md
@/Users/wilderherrera/Desktop/convision/.cursor/rules/convision-qa-explorer.mdc
@/Users/wilderherrera/Desktop/convision/.codex/get-shit-done/templates/QA-FINDINGS.md
</execution_context>

<context>
Argumentos: `{{GSD_ARGS}}`
- Vacío → los tres roles estándar (admin, specialist, receptionist).
- Un rol → solo ese rol (`admin`, `specialist`, `receptionist`).
- Texto extra → nota de alcance (priorizar módulos mencionados).
</context>

<process>
Ejecutar el workflow `qa-explore.md` de principio a fin, respetando prerequisitos, lectura obligatoria y plantilla de salida.
</process>
