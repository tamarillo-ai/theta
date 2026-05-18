# Project Guidelines - hongincanada.com

## Purpose
- Static personal portfolio and long-form writing site for Hong.
- No backend/API layer. Prefer declarative content and build-time rendering.

## Core Stack
- SvelteKit 2 + Svelte 5 (runes) + TypeScript.
- Tailwind CSS 3 with class-based dark mode and typography plugin.
- Node adapter deployment (`@sveltejs/adapter-node`).

## Ground Rules
- Use Svelte 5 runes patterns for new component logic.
- Preserve existing SEO and structured data on all edited pages.
- Keep animations performant and accessible (`prefers-reduced-motion`).
- Avoid heavy animation dependencies unless explicitly requested.
- Do not hand-edit generated output in `build/`.

## Canonical Source Areas
- Routes: `src/routes/`
- Shared article template: `src/lib/components/BaseSeriesPage.svelte`
- Series data: `src/lib/data/seriesData.ts`, `src/lib/data/mosaicSeriesData.ts`
- Shared types: `src/lib/types/series.ts`
- Global styles: `src/app.css`

## Editing Priorities
1. Protect content integrity and metadata correctness.
2. Preserve the dark-first, immersive design language.
3. Maintain responsive behavior across mobile/tablet/desktop.
4. Keep code simple and dependency-light.

## Validation Commands
```bash
npm run check
npm run build
```
