---
name: "type-check"
description: "Run TypeScript type checking via npm run type-check and present errors in a structured, actionable format. Use after editing TypeScript files, before committing, or when diagnosing type errors in HSM, AI loop, or memory layer code."
metadata:
  short-description: "Run tsc --noEmit and triage type errors by file and severity"
---

<objective>
Run the project's TypeScript compiler in check-only mode, parse the output,
and present errors grouped by file with fix suggestions.

This is the Codex equivalent of the typescript-lsp plugin used in Claude Code.
It uses the project's own tsconfig (strict mode, path aliases @/*) so all
diagnostics are accurate for this codebase.
</objective>

<context>
Arguments (optional — specific file or subsystem to focus on): {{GSD_ARGS}}

TypeScript config: tsconfig.json (strict, ES2022, path aliases @/*)
Check command: npm run type-check  (runs: tsc --noEmit)
</context>

<process>
1. **Run type checker**
   ```bash
   npm run type-check 2>&1
   ```

2. **Parse output**
   - If exit code 0 and no errors → report "✅ No type errors found." and stop.
   - If `{{GSD_ARGS}}` names a specific file or subsystem, filter errors to only that scope.

3. **Group and display errors**
   For each file with errors, show:
   ```
   📄 src/path/to/file.ts  (N errors)
     L42  TS2345  Argument of type 'X' is not assignable to parameter of type 'Y'
     L87  TS2339  Property 'foo' does not exist on type 'Bar'
   ```

4. **Prioritize**
   - **Critical (fix first):** errors in `src/hsm/machine.ts`, `src/ai/tools.ts`, `src/ai/loop.ts`
   - **High:** errors in any `*.primitive.ts`, `src/core/memory/`
   - **Normal:** everything else

5. **Suggest fixes** for each error group:
   - Missing types → check imports and path aliases (`@/types`, `@/hsm/types`)
   - XState v5 API mismatches → `setup({})`, `fromPromise()`, `assign()` patterns
   - `better-sqlite3` type errors → check `@types/better-sqlite3` version
   - `strict` null errors → add null guard or use optional chaining

6. **Summary line**
   ```
   Total: N errors across M files. Suggested fix order: [file list]
   ```
</process>
