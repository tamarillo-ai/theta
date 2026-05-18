# AGENTS.md

## Project

**AI-Powered Learning Assistant**

AI-Powered Learning Assistant is a public web app for readers, students, professors, and other knowledge-heavy users who want to study PDF documents with fast, modern AI assistance. It combines document upload and reading with AI chat, summaries, concept explainers, flashcards, quizzes, and progress tracking in a cleaner and more responsive experience than the typical AI study tool.

**Core value:** Make studying PDF documents feel fast, visually polished, and genuinely useful instead of slow, cluttered, and frustrating.

## Working Context

- Frontend lives in `frontend/`
- Backend lives in `backend/`
- Frontend stack: Next.js + Bun + Tailwind CSS + shadcn/ui
- Backend stack: Node.js + Express + MongoDB
- AI provider: Google Gemini
- Auth: JWT
- v1 focus: single-user, single-document study loop
- Deferred: team collaboration, multi-file chat, OCR-heavy support, native mobile

Full planning context lives in:

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/research/`

## Implementation Guidance

- Use MongoDB Atlas as both the application database and the first vector retrieval layer.
- Keep PDFs in durable object storage in production; do not rely on local disk.
- Keep Gemini access server-side only.
- Prefer document-grounded AI behavior over generic answers.
- Preserve the visual system with shadcn/ui instead of ad hoc one-off Tailwind components.

## GSD Workflow

Before making substantial repo changes, enter through a GSD workflow so planning artifacts stay aligned with execution.

Preferred entry points:

- `$gsd-discuss-phase 1` for Phase 1 context refinement
- `$gsd-ui-phase 1` for the frontend UI contract
- `$gsd-plan-phase 1` to turn Phase 1 into an executable plan
- `$gsd-execute-phase 1` after planning is approved

Git workflow:

- Use one git branch per phase before execution starts.
- Branching strategy is configured as `phase`.
- Phase branches use the template `gsd/phase-{phase}-{slug}`.
- Merge phase branches manually after review or when the milestone workflow calls for it.

Avoid making large direct edits without updating the planning artifacts when scope changes.
