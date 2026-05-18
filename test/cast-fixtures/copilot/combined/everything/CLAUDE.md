# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
npm run dev          # Start dev server
npm run build        # Production build
npm run preview      # Preview production build
npm run check        # Type-check with svelte-check
npm run lint         # Prettier check + ESLint
npm run format       # Auto-format with Prettier
```

There are no automated unit/integration tests in this project.

## Project Snapshot

**Stack:** SvelteKit 2 + Svelte 5 + TypeScript + Tailwind CSS, deployed via Node adapter.

This is a static content blog hosting two multi-part article series. No backend or API — all data is declarative TypeScript.

## Working Rules

- Prefer Svelte 5 runes patterns for new component logic.
- Preserve SEO metadata and JSON-LD structured data on edited pages.
- Keep motion performant and always respect `prefers-reduced-motion`.
- Avoid heavy animation dependencies unless explicitly requested.
- Do not hand-edit generated artifacts under `build/`.

## Validation Workflow

Run after meaningful changes:

```bash
npm run check
npm run build
```

### Routing

File-based SvelteKit routing:
- `/` — Home page
- `/series` — "How I Built MiniBreaks.io With AI" series landing (10 parts at `/series/part-{n}-{slug}`)
- `/mosaic` — "Mosaic: Rethinking App Design" series landing (4 parts at `/mosaic/{slug}`)

### Data Layer (Canonical)

All series metadata lives in `src/lib/data/`:
- `seriesData.ts` — AI development series (10 parts), exports `getPartData()`, `getNavigationData()`
- `mosaicSeriesData.ts` — Mosaic series (4 parts), exports `getArticleData()`

When adding a new series part, add its metadata to the relevant data file first, then create the route.

### Shared Component Pattern

`src/lib/components/BaseSeriesPage.svelte` is the shared template for all article pages. It handles:
- Breadcrumbs, article metadata, prev/next navigation
- Dark/light theme toggle (persisted in `localStorage`)
- Sticky TOC sidebar (Svelte `slide` transition)
- Full SEO: canonical URL, Open Graph, Twitter Card, JSON-LD `BlogPosting` schema

Individual article pages pass props to `BaseSeriesPage` and provide article content as a slot.

### Styling

- Tailwind with dark mode via `class` strategy
- Custom fonts: Inter (sans), Playfair Display (serif) from Google Fonts
- Prose typography via `@tailwindcss/typography`
- Custom CSS in `src/lib/app.css` for article heading styles (h2 = blue left border, h3 = green)

### Types

`src/lib/types/series.ts` defines the TypeScript interfaces used across both series data files and `BaseSeriesPage`.
