# Style guide

Conventions for theta CLI documentation.

## General

- Headings use sentence case
- Use sentence case in lists and descriptions
- No periods at the end of list items unless the item spans multiple sentences
- Unordered lists first — numbered lists **MAY** be used if enumeration is not replaceable
- One idea per paragraph
- Use backticks for: commands, code expressions, field names, file paths, package names, environment variables
- Use `->` or `-->` for arrows, never `→` or other ligatures
- Prefer md links over bare URLs: `[name](url)`
- Math ligatures **MUST** be avoided
- Never use emojis
- Never use ASCII separator banners (`// -------`, `// === foo ===`, `// -- section --`, etc.) in code or docs — structure code with functions and structure prose with headings instead

## RFC 2119

Keywords (MUST, MUST NOT, SHOULD, SHOULD NOT, MAY) carry normative meaning per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

- Capitalize them
- Make them bold
- Do not explain what they mean
- Introduce only once the reference in documentation sections where these words are used

## Structure

- **Headings carry the architecture** — don't restate the heading in the first sentence
- **Tables over paragraphs** for structured data (fields, mappings, comparisons)
- **Code blocks** for anything the user might copy — **MUST** use language markers (`toml`, `bash`, `json`, etc.)
- **Links over explanation** — if documented elsewhere, link to it instead of restating

## Code blocks

- All code blocks **MUST** have a language marker
- Command examples use `bash` (not `console`)
- Command output **SHOULD** rarely be included — hard to keep current

## Admonitions

Use [mkdocs-material admonitions](https://squidfunk.github.io/mkdocs-material/reference/admonitions/):

- `note` — supplementary context
- `warning` — something that may affect you (e.g. data loss, breaking changes, lossy casting)
- `info` — harness-specific notes, compatibility details
- `tip` — practical recommendation
- `danger` — constraint violation, security concern

Titles **MUST** be informative, not generic.

## Cross-references

- Every reference to another doc, concept, or field **SHOULD** be a markdown link
- External references **MUST** include the full URL
- Internal links use relative paths
- Link to [theta-spec](https://theta-spec.tamarillo.ai/) for field definitions, not restate them
