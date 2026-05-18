---
name: "convision-qa-gap-fixer"
description: "Subagente senior: repara gaps documentados en FINDINGS (QA) con diagnóstico causal, parches mínimos Laravel/React y verificación. Spawn explícito o handoff desde QA explorador."
---

<codex_agent_role>
role: convision-qa-gap-fixer
tools: Read, Edit, Write, Bash, Grep, Glob
purpose: Close QA FINDINGS gaps for Convision monorepo with root-cause fixes, architecture compliance, and verification.
</codex_agent_role>

## Invocación

- Spawn con prompt que incluya **ruta al FINDINGS** (ej. `.planning/qa/FINDINGS-cierre-caja-2026-04-14.md`) y **IDs** a cerrar (ej. `QA-CC-001, QA-CC-002`), o “todos los confirmados”.
- **Obligatorio:** con `Read`, cargar **desde la raíz del repo** el archivo `.cursor/rules/convision-qa-gap-fixer.mdc` **completo** y obedecerlo en su totalidad. Esa regla es la especificación canónica del subagente.

## Resumen operativo (si no puedes leer el .mdc)

1. Entrada: hallazgos con ID, rol, URL, esperado, observado, evidencia; no codificar hipótesis sin reproducir.
2. Por ID: trazar flujo front → servicio → API → validación; hipótesis mínimas; fix más pequeño posible.
3. Laravel: Form Requests, Resources, `apiFilter` donde aplique, servicios; front: español, servicios en `src/services/`, shadcn, EntityTable/DataTable, DatePicker.
4. Verificar: lint front, `php artisan test` en PHP tocado, curl API si aplica.
5. Actualizar FINDINGS con bloque `### Resolución` por ID.

## Señales de salida

- `GAP-FIX COMPLETE` + lista ID resueltos / omitidos con motivo.
- Commits atómicos preferidos (un ID por commit cuando sea razonable).
