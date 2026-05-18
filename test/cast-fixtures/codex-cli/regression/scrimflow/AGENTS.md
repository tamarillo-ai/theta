# AGENTS.md

Single source of truth for agent guidance in this repository. If scripts, architecture, or workflow change, update this file and keep `CLAUDE.md` as `@AGENTS.md` only.

## Read This First

- Verify current repo state from code before making assumptions. Check `package.json`, app/package configs, and the relevant source files.
- Before any Next.js work, read the relevant doc in `apps/web/node_modules/next/dist/docs/`.
- Start with `apps/web/node_modules/next/dist/docs/index.md`, then read the specific doc for the feature you are changing.
- This repo is on Next `16.2.1-canary.10` and React `19.2.3`; older Next habits are not reliable here.

## Operating Rules

- Use `pnpm` only.
- Use Biome, not ESLint or Prettier.
- Use Valibot, not Zod.
- Use `@hugeicons/react`, not `lucide-react` or Heroicons.
- Add shadcn/ui components from `apps/web/`.
- Prefer existing shared contracts from `@scrimflow/shared` before inventing new types or route strings.
- Do not add compatibility redirects, legacy route aliases, dead code, or unused routes for internal app paths. This project has never launched, so obsolete paths should be removed outright instead of preserved.
- After code changes, run the smallest relevant verification command and report what you ran.

## Repo Snapshot

- Product: Overwatch 2 team management platform.
- Monorepo packages:
  - `apps/web` - Next.js App Router frontend
  - `packages/api` - Hono API running on Bun
  - `packages/shared` - shared DTOs, validations, permissions, routes, and config
- Tooling:
  - Next `16.2.1-canary.10`
  - React `19.2.3`
  - Tailwind CSS `4`
  - Biome `2.3.15`
  - Drizzle ORM
  - PostgreSQL, Redis, MinIO, Mailpit
- Workspace config: `pnpm-workspace.yaml` includes `apps/*` and `packages/*`.

## Current Commands

Run these from the repo root unless a command explicitly says otherwise.

### Setup

```bash
cp .env.example .env
docker compose -f docker-compose.dev.yml up -d
pnpm install
```

### Development

```bash
pnpm dev
pnpm dev:web
pnpm dev:api
pnpm dev:worker
```

### Quality

```bash
pnpm check
pnpm check:fix
pnpm lint
pnpm lint:fix
pnpm format
pnpm format:fix
```

### Database

```bash
pnpm db:generate
pnpm db:migrate
pnpm db:push
pnpm db:studio
pnpm db:seed
```

### Notes

- Root scripts load `.env` with `dotenv-cli`.
- Pre-commit runs `pnpm exec biome check` on staged JS/TS/JSON files through Lefthook.
- There is no dedicated automated test suite configured at the root right now. Default verification is Biome plus targeted manual or route-level validation.

## Product Surface Today

The live app is broader than auth and a basic workspace shell. Current areas include:

- Public marketing and discovery pages under `apps/web/app/(home)`:
  - landing page
  - public org pages
  - public team pages
  - public player pages
  - recruiting, updates, scrims, about, contact, privacy, terms
- Auth flow under `apps/web/app/(auth)/auth`:
  - single-page step router
  - login, register, forgot password, verify email
  - new-device verification
  - TOTP, passkeys, security keys, recovery codes
- Onboarding under `apps/web/app/onboarding`:
  - battletag
  - roles and rank
  - hero pool
- App workspace under `apps/web/app/app`:
  - personal home, inbox, calendar, profile, settings
  - team overview, roster, calendar, scrims, chat, recruiting, updates, settings
  - org overview, teams, staff, brand, settings
  - deletion-pending still exists as a separate gated route, but all authenticated workspace navigation should land in `/app`

## Architecture

### Web App

- App Router lives in `apps/web/app`.
- Route groups: `(auth)`, `(home)`, `app`, `onboarding`, `deletion-pending`.
- `apps/web/proxy.ts` handles fast edge-level cookie gating for protected routes.
- `apps/web/app/app/layout.tsx` is the authenticated workspace shell and should remain the only primary authenticated app surface.
- The frontend does not access the database directly. It talks to the API over HTTP.
- `apps/web/next.config.ts` rewrites `/api/:path*` to `API_URL`.
- Server-side API access is centralized in `apps/web/lib/api-client.ts`, which forwards cookies and auth `Set-Cookie` headers.
- Web-side mutations should use `apps/web/lib/api-client.ts` plus shared route/contracts. There is no separate client SDK package in this repo anymore.
- Scrim evidence uploads now use a direct-to-object-storage flow: request an upload intent from the API, upload with the signed URL, finalize the object, then create the OCR job.
- Client-side scrim OCR panels refresh against the API and should stay aligned with the shared OCR job contract.
- Scrim OCR is screenshot-specific: `game_history` jobs extract visible match rows and can prefill reviewed scrim result maps, while `scoreboard` jobs extract ally/enemy player stat rows and stay evidence-only until a dedicated merge flow exists.
- The reviewed scrim result editor can now import a completed scoreboard OCR job into a selected reviewed map, replacing that map's player rows and recording the scoreboard job as map-level supporting evidence in the revision snapshot.
- Reviewed scrim result submission now persists real `scrim_map` and `scrim_player_stat` rows. The client-side scrim result editor can stay series-only, or load an OCR draft and submit reviewed maps/player stats as the authoritative result package.
- Reviewed scrim result submissions now also append immutable `scrim_result_revision` snapshots with structured correction diffs. The scrim detail page is the canonical place to inspect that revision history.
- Client-side realtime websocket helpers live in `apps/web/lib/ws`.
- Team chat now has its own `/app/teams/[teamId]/chat` workspace with a persistent team room plus scrim-linked channels.
- The `/app` shell now keeps inbox unread state live through the app realtime websocket; `/app/inbox` should be treated as the primary inbox surface.
- Team updates now live under `/app/teams/[teamId]/updates` and use team-scoped realtime events for live feed changes.
- Team overview, roster, calendar, and settings now live directly under `/app/teams/[teamId]`; pending team invites should be surfaced from the roster workspace rather than a separate app-native invites route.
- Team overview now surfaces recent rating history, and completed scrims now surface their applied rating changes directly in the scrim workspace.
- Disputed scrims now resolve through explicit org-level review. Use `/api/scrims/:id/resolve-dispute` to either finalize the reported result or void the scrim; once results have been reported, generic scrim cancellation is no longer the settlement path.
- Recruiting is app-native under `/app/recruiting` and `/app/recruiting/conversations`; do not recreate `/app/recruiting/posts` or any other compatibility alias.
- Org workspace pages under `/app/orgs/[orgId]` are now app-native; treat `/app/orgs/[orgId]/brand` as the profile and media surface, and `/app/orgs/[orgId]/settings` as ownership and danger-zone management.
- Personal profile and personal account/security settings under `/app/profile` and `/app/settings/*` are app-native; do not recreate any legacy personal route split.
- The `/app` shell now owns its own `layout.tsx`, `loading.tsx`, and `error.tsx`; shared workspace auth/onboarding/bootstrap logic lives in `apps/web/lib/workspace-shell.ts`.
- User-facing workspace navigation should use `/app` routes consistently: org creation/listing belongs under `/app/orgs`, settings links should target `/app/settings/*`, and generic profile/home shortcuts should stay inside the `/app` tree.
- Active workspace components should import server actions from `apps/web/app/actions/*`; the removed legacy workspace tree must not be recreated.

### API

- API entrypoint: `packages/api/src/index.ts`.
- Runtime: Bun.
- OCR worker entrypoint: `packages/api/src/worker/index.ts`.
- Hono routes currently mounted for:
  - auth
  - settings
  - profile
  - onboarding
  - orgs
  - teams
  - realtime websocket
  - recruitment listings
  - recruitment applications
  - updates
  - chat
  - schedule
  - notifications
  - uploads
  - users
  - public orgs, teams, players, recruitment listings
  - heroes
- Auth-required routes are protected in the API with `requireAuth`.
- Upload routes now include signed scrim evidence intent and finalize endpoints alongside legacy asset uploads.
- OCR worker contracts are screenshot-specific; do not collapse `game_history` and `scoreboard` extraction back into one generic payload.
- Chat routes now expose team-scoped and scrim-scoped conversation listings; pending scrims create negotiation channels and accepted scrims create lobby channels.
- App realtime routes now carry both scrim OCR events and user-scoped inbox notification events.
- Updates routes expose authenticated team/org update feeds at `/api/updates` and public feed reads at `/api/public/updates`.
- Once both teams confirm a scrim result, the API now applies team rating changes, writes immutable `team_rating_event` rows, and treats the completed scrim as locked from further confirmation edits.

### Shared Package

- `packages/shared/src/types.ts` is the main shared contract surface.
- `packages/shared/src/routes.ts` defines `apiRoutes`, `appRoutes`, and `publicRoutes`.
- `packages/shared/src/permissions.ts` contains role/permission helpers.
- Validation schemas live in `packages/shared/src/validations`.

## Important Domain Conventions

- Auth UI state lives in Zustand:
  - `apps/web/stores/auth-flow.ts`
  - `apps/web/stores/onboarding-flow.ts`
- Auth is step-based, not route-per-step.
- Session token is stored in the `session_token` HttpOnly cookie.
- Web session reads go through `/api/auth/session`, not direct cookie parsing for full validation.
- Redis-backed rate limiting exists with an in-memory fallback.
- TOTP secrets and recovery codes are encrypted using `ENCRYPTION_KEY`.
- Shared DTO convention: use transport-safe strings such as ISO date strings across boundaries and parse only where needed.

## Current Data Model Shape

The database schema is already wider than the original MVP. Core entities include:

- users and sessions
- player profiles, heroes, maps
- organizations and organization members
- teams and team rosters
- org and team invites
- recruitment listings, applications, and conversation threads
- availability and scrim scheduling
- notifications
- chat channels and messages
- OCR/scrim-related tables are present in schema and migrations, even if some product areas are still maturing

## UI Conventions

- shadcn/ui is configured in `apps/web/components.json` with:
  - style: `radix-lyra`
  - RSC enabled
  - Hugeicons as the icon library
- Shared UI primitives live in `apps/web/components/ui`.
- Reuse existing shells and sections before creating new layout patterns:
  - workspace shell
  - page container/header/section
  - auth shell
  - onboarding shell

## Environment And Services

- Use one root `.env` file for the whole repo.
- Key local services from `docker-compose.dev.yml`:
  - PostgreSQL
  - Redis
  - MinIO
  - Mailpit
  - Redis Commander
- Important environment variables:
  - `DATABASE_URL`
  - `API_URL`
  - `NEXT_PUBLIC_APP_URL`
  - `GEMINI_API_KEY`
  - `GEMINI_MODEL`
  - `OCR_WORKER_*`
  - `REDIS_URL`
  - `ENCRYPTION_KEY`
  - `S3_*`
  - `WEBAUTHN_RP_ID`
  - `WEBAUTHN_ORIGIN`
  - `SMTP_*`

## Agent Workflow

- For Next.js changes:
  - read the relevant file in `apps/web/node_modules/next/dist/docs/`
  - inspect the existing route/component/helper before coding
  - preserve App Router and Server Component patterns already in use
- For shared API or route changes:
  - update `packages/shared` contracts first or alongside the change
  - keep `apps/web` route usage aligned with `packages/shared/src/routes.ts`
- For validation changes:
  - update Valibot schemas in `packages/shared/src/validations`
  - do not introduce ad hoc validation libraries
- For new UI:
  - match the current design system and existing shells
  - prefer existing `components/ui` and shared components over bespoke primitives

## File Map

- `apps/web/app` - routes, layouts, error boundaries, server actions
- `apps/web/components` - UI and domain components
- `apps/web/lib` - API client, auth/session helpers, data fetchers, config
- `apps/web/stores` - Zustand client stores
- `packages/api/src/routes` - Hono route handlers
- `packages/api/src/auth` - session, password, 2FA, WebAuthn, security logic
- `packages/api/src/db/schema` - Drizzle schema
- `packages/shared/src` - shared DTOs, routes, permissions, validations

## Avoid These Mistakes

- Do not add ESLint, Prettier, Zod, or alternate icon libraries.
- Do not hardcode app or API paths when shared route helpers already exist.
- Do not bypass `apps/web/lib/api-client.ts` for standard web-to-API calls.
- Do not keep legacy redirects, compatibility aliases, dead code, or unused routes just to preserve old internal behavior.
- Do not describe the project as auth-only; recruiting, org/team management, schedule, notifications, and public pages are already part of the current product state.
