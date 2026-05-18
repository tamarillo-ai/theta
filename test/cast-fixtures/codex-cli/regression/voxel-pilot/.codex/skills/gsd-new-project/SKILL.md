---
name: "gsd-new-project"
description: "Use when a repository is not yet initialized in GSD and you need to turn an idea or existing codebase into concrete planning artifacts before phase work starts."
metadata:
  short-description: "Initialize a not-yet-planned project into PROJECT.md, config, requirements, roadmap, and state artifacts."
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-new-project`.
- Treat all user text after `$gsd-new-project` as `{{GSD_ARGS}}`.
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

<overview>
Initialize a project through the full GSD setup flow: clarify the idea, capture durable project context, optionally research the domain, define scoped requirements, and generate a roadmap that phase workflows can execute.

This is not a scaffold command and not a shallow intake form. The quality of `PROJECT.md`, `REQUIREMENTS.md`, and `ROADMAP.md` determines whether every downstream phase is guided or forced to guess.
</overview>

<when_to_use>
Use this skill when:
- the repository has not been initialized with GSD artifacts yet
- the user has an idea but not a concrete planning baseline
- a greenfield project needs proper questioning before planning
- a brownfield codebase needs existing capabilities separated from new scope
- you need `.planning/` artifacts that make `/gsd-discuss-phase 1` or `/gsd-plan-phase 1` possible

Do not use this skill when:
- `.planning/PROJECT.md` already exists and the project is active — use `/gsd-progress` or the phase workflows instead
- the real need is brownfield architecture discovery first — run `/gsd-map-codebase` before continuing when existing code is detected and unmapped
- the user only wants to tweak requirements or roadmap for an already initialized project
</when_to_use>

<operator_stance>
Treat project initialization as ambiguity removal. The job is not to “get to roadmap quickly.” The job is to make downstream planning obvious.

If the user gives fuzzy language, challenge it. If the user gives a broad dream, narrow it. If the codebase already exists, distinguish validated reality from desired future work. A vague initialization poisons every later command.
</operator_stance>

<quick_reference>
| Topic | Guidance |
|------|----------|
| Primary input | Freeform project idea, pasted brief, or `--auto` with document content |
| Brownfield rule | Existing code without a codebase map should trigger a `/gsd-map-codebase` offer first |
| Auto mode | `--auto` requires an idea document or explicit written brief; skip deep questioning, not quality |
| Core outputs | `.planning/PROJECT.md`, `.planning/config.json`, optional `.planning/research/`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`, `AGENTS.md` |
| Research role | Research is optional, but when selected it must materially shape requirements and roadmap |
| Success bar | Every v1 requirement maps to exactly one phase and Phase 1 can be discussed/planned immediately |
</quick_reference>

<quality_bar>
Initialization is only complete when all of the following are true:
- `PROJECT.md` explains what the product is, why it exists, who it is for, and what “done” means in concrete terms
- Active requirements are hypotheses, not vague aspirations
- Existing capabilities in brownfield repos are recorded as validated, not mixed into speculative scope
- `REQUIREMENTS.md` contains atomic, user-centric, testable requirements with stable REQ-IDs
- `ROADMAP.md` covers 100% of v1 requirements with explicit phase goals and observable success criteria
- the user knows the next command to run and why

If any of these are weak, initialization is not done. It is a draft pretending to be a plan.
</quality_bar>

<decision_points>
Key judgment calls during execution:

1. **Greenfield vs brownfield**
If code already exists, do not treat the repo like a blank slate. Infer current validated behavior from the codebase or insist on mapping first.

2. **Questioning depth**
Do not stop when you merely understand the noun. Stop when you understand the motivation, boundaries, and concrete shape of the product well enough to write a durable `PROJECT.md`.

3. **Research necessity**
Research is worth the tokens when the domain has strong conventions, major architecture choices, or costly pitfalls. Skip it only when the space is already well understood or the project is intentionally tiny.

4. **Requirement scoping**
Do not let “v1” become an aspiration bucket. Table stakes omitted from v1 should be explicitly tracked as deferred or out of scope, not silently lost.

5. **Roadmap approval**
A roadmap is not approved because it looks plausible. It is approved when phase boundaries are coherent, requirement coverage is complete, and success criteria are observable.
</decision_points>

<context>
`{{GSD_ARGS}}` may contain a phase-free project idea plus flags.

**Flags:**
- `--auto` — Automatic mode. After config questions, runs research → requirements → roadmap without further interaction. Requires an idea document by `@file` reference or explicit written brief.

**Important distinctions:**
- `--auto` skips conversational questioning; it does not lower the quality bar.
- Brownfield repos still require correct separation between validated current state and new desired scope.
- Missing or weak source material in `--auto` mode is an error, not a cue to improvise.
</context>

<execution_context>
@D:/Developer/Minecraft_bot/.codex/get-shit-done/workflows/new-project.md
@D:/Developer/Minecraft_bot/.codex/get-shit-done/references/questioning.md
@D:/Developer/Minecraft_bot/.codex/get-shit-done/references/ui-brand.md
@D:/Developer/Minecraft_bot/.codex/get-shit-done/templates/project.md
@D:/Developer/Minecraft_bot/.codex/get-shit-done/templates/requirements.md
</execution_context>

<runtime_note>
**Copilot (VS Code):** Use `vscode_askquestions` wherever this workflow calls `AskUserQuestion`. They are equivalent — `vscode_askquestions` is the VS Code Copilot implementation of the same interactive question API.
</runtime_note>

<process>
Execute the new-project workflow from @D:/Developer/Minecraft_bot/.codex/get-shit-done/workflows/new-project.md end-to-end.

Non-negotiable execution rules:
- Read the workflow file before acting. This `SKILL.md` is the operator guide; the workflow is the source of procedural truth.
- Preserve workflow gates. Do not compress questioning, approval, or validation steps just to move faster.
- Use `questioning.md` to follow the user's thread instead of interrogating from a checklist.
- Treat `PROJECT.md` as a living project context document, not a one-time summary.
- Treat `REQUIREMENTS.md` as a scoping contract for roadmap creation; vague items must be sharpened before roadmaping.
- Treat `ROADMAP.md` as executable decomposition. Every v1 requirement must land in exactly one phase.
- Generate or refresh `AGENTS.md` before the final roadmap commit so the project instructions reflect the initialized state.
</process>

<brownfield_guidance>
Brownfield handling is where weak operators usually fail.

Rules:
- If existing code is detected and no codebase map exists, offer `/gsd-map-codebase` first.
- Infer current capabilities from the codebase and place them in `Validated`, not `Active`.
- Only the newly requested work belongs in `Active`.
- Do not overwrite reality with the user's wish list. Separate “what the repo already does” from “what the user wants next”.
</brownfield_guidance>

<questioning_guidance>
During interactive mode:
- start open: let the user explain the product in their own words
- follow what they emphasize instead of forcing a canned interview
- challenge vague language immediately
- make abstractions concrete with examples, flows, and “what does this actually look like?”
- stop only when you can write `PROJECT.md` without padding it with generic boilerplate

The goal is not more questions. The goal is better questions that eliminate future guesswork.
</questioning_guidance>

<auto_mode_guidance>
In `--auto` mode:
- require a real idea document or sufficiently detailed written brief
- collect config first, because settings affect how the rest of the workflow runs
- synthesize requirements from the source material plus domain table stakes
- defer differentiators not supported by the source material rather than inventing them
- auto-approve only after quality checks pass internally

`--auto` is for reducing interaction, not for reducing rigor.
</auto_mode_guidance>

<anti_patterns>
Reject these failure modes:
- writing a generic `PROJECT.md` that could describe any startup
- jumping to stack or architecture before understanding the product
- treating “users can do stuff” as acceptable requirement language
- mixing current brownfield behavior with future desired scope
- approving a roadmap that does not show full requirement coverage
- skipping research because “it probably works like other apps”
- using `--auto` without enough source material, then filling gaps with guesses
</anti_patterns>

<objective>
Initialize a new project through unified flow: questioning → research (optional) → requirements → roadmap.

**Creates:**
- `.planning/PROJECT.md` — project context
- `.planning/config.json` — workflow preferences
- `.planning/research/` — domain research (optional)
- `.planning/REQUIREMENTS.md` — scoped requirements
- `.planning/ROADMAP.md` — phase structure
- `.planning/STATE.md` — project memory

**After this command:** Run `/gsd-discuss-phase 1` to shape Phase 1 context, or `/gsd-plan-phase 1` if discussion can be skipped safely.
</objective>

<success_criteria>
- `.planning/` exists and contains the expected initialization artifacts
- git initialization and doc commit behavior match config decisions
- brownfield detection was handled correctly instead of ignored
- `PROJECT.md` contains concrete product context rather than generic filler
- `REQUIREMENTS.md` is scoped, testable, and traceable
- `ROADMAP.md` has coherent phases, full v1 coverage, and observable success criteria
- `STATE.md` and `AGENTS.md` are aligned with the initialized project state
- the user leaves with a clear next step, not just a pile of files
</success_criteria>
