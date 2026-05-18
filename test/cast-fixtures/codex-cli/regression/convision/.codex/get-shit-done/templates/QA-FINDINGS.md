# Plantilla QA-FINDINGS.md

Archivo de salida recomendado: `.planning/qa/FINDINGS-YYYY-MM-DD.md` (o ruta que indique el usuario).

---

## Frontmatter (opcional)

```yaml
---
status: in_progress | complete
app: convision-front
api: convision-api
base_url: http://localhost:4200
started: [ISO-8601]
updated: [ISO-8601]
roles_tested: [admin, specialist, receptionist]
---
```

## Resumen ejecutivo

- Pantallas verificadas: N
- Hallazgos confirmados: N
- Hipótesis / pendiente evidencia: N
- Sin incidencias (lista): …

## Hallazgos (FAIL / GAP)

Por cada ítem usar bloque:

```text
### QA-001
- Rol: admin
- URL: http://localhost:4200/admin/...
- Severidad: bloqueante | mayor | menor | sugerencia
- Pasos: 1. … 2. …
- Esperado: …
- Observado: …
- Evidencia: (mensaje UI / HTTP / consola)
- Estado: confirmado | hipótesis
```

## OK (sin incidencias)

Solo listar rutas realmente abiertas y comprobadas en esta sesión:

| Rol | Ruta | Notas |
|-----|------|--------|
| admin | /admin/dashboard | Carga, sin error consola |

## Handoff al agente de corrección

- **Recomendado (subagente potente):** regla Cursor `convision-qa-gap-fixer` o agente Codex `convision-qa-gap-fixer` — incluir ruta de este archivo + IDs.
- **Ligero:** regla `convision-qa-fixer`.
- Comando sugerido: “Con `@convision-qa-gap-fixer`, cerrar QA-XXX… usando este FINDINGS como fuente.”
