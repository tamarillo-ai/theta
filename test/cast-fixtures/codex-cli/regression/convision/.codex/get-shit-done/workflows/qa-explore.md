<purpose>
Exploración QA funcional **automatizada con navegador** sobre la SPA Convision: login por rol, recorrido del menú lateral y rutas del mapa, red/consola, salida estructurada para un agente corrector.

**Diferencia vs verify-work:** `verify-work` es UAT **conversacional** con humano (preguntas sí/no por fase). `qa-explore` es **agente + MCP navegador** sin depender de `.planning/phases`; alimenta hallazgos en `.planning/qa/` o `docs/`.
</purpose>

<required_reading>
Antes de ejecutar pasos, leer en el repo:

- `docs/QA_MAPA_EXPLORACION.md` — rutas, roles, menú en español, URLs sin sidebar.
- `docs/CREDENCIALES_PRUEBA_ROLES.md` — usuarios seed (solo local).
- `.cursor/rules/convision-qa-explorer.mdc` — conducta del agente explorador.
</required_reading>

<prerequisites>
- API Laravel en `http://localhost:8000`, front Vite en `http://localhost:4300` (ver `convision-front/vite.config.ts`).
- Herramientas MCP de navegador disponibles en la sesión (p. ej. cursor-ide-browser: navigate, snapshot, interacciones; tras cada cambio de pantalla, **nuevo snapshot** antes de la siguiente acción).
- No ejecutar borrados destructivos ni datos reales; entorno local con seeders.
</prerequisites>

<test_users>
Usuarios para login en `/login` (email + password). Contraseña común: **`password`**. Detalle y actualizaciones: `docs/CREDENCIALES_PRUEBA_ROLES.md`.

**Genéricos (`UsersTableSeeder`):**

| Rol (API)        | Correo                       |
|------------------|-----------------------------|
| admin            | `admin@convision.com`       |
| specialist       | `specialist@convision.com`  |
| receptionist     | `receptionist@convision.com`|

**Demo (`DemoStaffSeeder`):**

| Rol (API)        | Correo                     |
|------------------|----------------------------|
| admin            | `cvargas@convision.com`    |
| specialist       | `abermudez@convision.com`  |
| specialist       | `storres@convision.com`    |
| specialist       | `dmontoya@convision.com`   |
| receptionist     | `vcastillo@convision.com`  |
| receptionist     | `jnieto@convision.com`     |
| laboratory       | `hquintero@convision.com`  |

**Login API (JWT):** `POST http://localhost:8000/api/v1/auth/login` con JSON `{"email":"…","password":"password"}`.
</test_users>

<template>
@/Users/wilderherrera/Desktop/convision/.codex/get-shit-done/templates/QA-FINDINGS.md
</template>

<args>
`{{GSD_ARGS}}` (opcional):

- Vacío → probar los tres roles: `admin`, `specialist`, `receptionist` (flujo completo del mapa para el alcance razonable en una sesión).
- `admin` | `specialist` | `receptionist` → solo ese rol.
- Cualquier texto adicional → tratar como nota de alcance (p. ej. “solo COMERCIAL”) y priorizar esas secciones del mapa.
</args>

<process>

<step name="bootstrap">
1. Crear directorio `.planning/qa/` si no existe.
2. Elegir archivo de salida: `.planning/qa/FINDINGS-$(date +%Y-%m-%d).md` o continuar append si el usuario indica un archivo existente.
3. Inicializar el documento con la plantilla QA-FINDINGS (frontmatter + secciones vacías).
</step>

<step name="per_role">
Por cada rol en el alcance:

1. Ir a `http://localhost:4300/login`.
2. Rellenar correo y contraseña según credenciales del doc (campo email + password; botón **Ingresar**).
3. Confirmar redirección al dashboard esperado del rol.
4. Si el login falla o redirige a `/unauthorized`, registrar hallazgo con evidencia (URL final, mensaje UI, red).
5. Recorrer **cada ítem del sidebar** visible (clic en botones con texto en español del mapa).
6. En cada pantalla: esperar carga; revisar errores visibles; opcional `browser_network_requests` / `browser_console_messages` si hay fallo o vacío sospechoso.
7. Recorrer **rutas del mapa que no están en el menú** relevantes al rol (tabla “Rutas útiles que no aparecen en el menú lateral” en `docs/QA_MAPA_EXPLORACION.md`).
8. Cerrar sesión o borrar estado si hace falta para cambiar de rol (logout desde footer/layout según UI).

**Usuario laboratory (seed):** si se pide explícitamente o hay tiempo, probar `hquintero@convision.com` y documentar si termina en `/unauthorized` (gap conocido front vs API).
</step>

<step name="iterate">
- Máximo **un** reintento por pantalla con snapshot fresco si falla interacción.
- No más de 4 intentos ciegos en la misma vista; si bloquea, documentar **bloqueado por automatización** con última evidencia.
- Marcar hallazgos **confirmado** solo con evidencia; si no hay evidencia clara → **hipótesis**.
</step>

<step name="finalize">
1. Completar secciones Resumen, FAIL/GAP, OK del archivo FINDINGS.
2. Listar “Handoff” con IDs para el agente que use la regla `convision-qa-fixer`.
3. Si el usuario quiere integración GSD por fase: opcionalmente copiar un resumen de IDs críticos a un todo: `/gsd-add-todo`.
</step>

</process>

<anti_patterns>
- No afirmar “todo OK” sin listar rutas comprobadas.
- No editar código en este workflow (solo documentación de hallazgos salvo que el usuario ordene lo contrario).
- No asumir Playwright-MCP; usar las herramientas de navegador **realmente disponibles** en la sesión.
</anti_patterns>
