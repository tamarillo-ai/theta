#!/usr/bin/env node
/**
 * Codex local install fallback hook.
 *
 * The upstream installer configures a SessionStart hook for Codex but does not
 * always materialize hook files under .codex/hooks. Keep this lightweight no-op
 * to prevent missing-file hook errors while preserving normal GSD operation.
 */
process.exit(0);
