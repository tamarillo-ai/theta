---
description: "Use when editing long-form article pages or shared series templates. Covers floating/sticky TOC behavior, heading anchors, accessibility, and SEO safety requirements."
applyTo: "src/routes/series/**/*.svelte, src/routes/mosaic/**/*.svelte, src/lib/components/BaseSeriesPage.svelte"
---

# Article Page Rules

- Keep long-form reading flow first; navigation should support, not interrupt, reading.
- Keep desktop TOC sticky/floating and mobile TOC collapsed by default unless request says otherwise.
- Preserve heading anchor behavior and scroll offsets so in-page links do not hide behind sticky bars.
- Preserve SEO metadata and JSON-LD schema fields.
- Preserve series prev/next navigation behavior.
- Respect `prefers-reduced-motion` for TOC and related transitions.
- Validate with `npm run check` and `npm run build` after meaningful article-template changes.
