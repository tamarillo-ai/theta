# AGENTS — Snacks 911

## 🧠 Sistema de agentes

El sistema utiliza un framework GSD ubicado en:

.agent/skills/

---

## 🌟 Agentes principales

- gsd-autonomous → control completo del sistema
- gsd-manager → administración de fases
- gsd-plan-phase → planificación
- gsd-execute-phase → ejecución
- gsd-progress → estado del sistema
- gsd-audit-fix → detección y corrección de errores
- gsd-map-codebase → análisis de arquitectura

---

## 🛠️ Tipos de habilidades

### 1. Planeación
- gsd-plan-phase
- gsd-plan-milestone-gaps

### 2. Ejecución
- gsd-execute-phase
- gsd-do

### 3. Auditoría
- gsd-audit-fix
- gsd-code-review

### 4. Gestión
- gsd-manager
- gsd-progress

### 5. Investigación
- gsd-research-phase
- gsd-intel

---

## 🔁 Flujo del sistema

idea  
→ planificación  
→ ejecución  
→ auditoría  
→ deploy  

---

## 🔗 Integraciones

- GitHub Issues → tracking
- Supabase → datos
- Next.js → app
- Multi-AI → decisiones

---

## ⚠️ Reglas

- No ejecutar sin plan
- No crear código sin contexto
- Todo debe ser trazable