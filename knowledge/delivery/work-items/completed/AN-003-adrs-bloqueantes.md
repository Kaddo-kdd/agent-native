---
id: AN-003
type: chore
status: completed
completed_at: 2026-10-17
title: Materializar ADRs bloqueantes (001..005)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - knowledge/tech/decisions/ADR-001-arquitectura-inicial.md
  - knowledge/tech/decisions/ADR-002-runtime-independiente.md
  - knowledge/tech/decisions/ADR-003-semantic-ir-v05-baseline.md
  - knowledge/tech/decisions/ADR-004-seguridad-minima-transversal.md
  - knowledge/tech/decisions/ADR-005-modelo-memoria-canonical-abi.md
  - knowledge/tech/discovery/decision-candidates.md
source_roadmap_initiative: INIT-01-03
source_work_item_candidate:
  - WI-CANDIDATE-01-03-01
  - WI-CANDIDATE-01-03-02
  - WI-CANDIDATE-01-03-03
  - WI-CANDIDATE-01-03-04
  - WI-CANDIDATE-01-03-05
related_domain: Foundation & Governance
domains:
  - Foundation & Governance
related_capabilities: []
expected_value: >-
  Decisiones arquitectónicas bloqueantes materializadas como ADR drafts
  firmables por humano, con contexto, opciones consideradas y consecuencias
  listas.
risks:
  - >-
    R-ADR-01: materializar como accepted sin revisión humana — mitigado dejando
    todas como draft
  - >-
    R-ADR-02: fijar baseline técnica (p. ej. versión exacta de Wasmtime) sin
    evidencia — mitigado dejando [open] en ADR-003 hasta validar
dependencies:
  - AN-001
  - AN-002
source_signals:
  - >-
    Tech Decision Candidate: 5 candidatos bloqueantes en
    knowledge/tech/discovery/decision-candidates.md
  - >-
    Risk: iniciar INIT-01-01 sin decisiones aprobadas lleva a workspace sobre
    supuestos
decision_candidates:
  - ADR-001 Arquitectura inicial
  - ADR-002 Runtime independiente
  - ADR-003 Modelo IR v0.5
  - ADR-004 Seguridad mínima transversal
  - ADR-005 Modelo de memoria
related_decisions: []
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-17'
---

# AN-003 — Materializar ADRs bloqueantes (001..005)

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

**Actor and outcome:**
Equipos y agentes disponen de los 5 ADRs bloqueantes de RM-01 como drafts materializados en `knowledge/tech/decisions/`, cada uno con contexto, opciones, decisión propuesta y consecuencias. El humano acepta cada uno de forma individual (`status: accepted`) tras revisión.

**Current behavior:**
`knowledge/tech/discovery/decision-candidates.md` lista los 5 candidatos con contexto. `knowledge/tech/decisions/` está vacío. INIT-01-01 (workspace) depende de que ADR-001 y ADR-003 estén aprobados antes de abrirlo.

**Target behavior:**
- 5 archivos bajo `knowledge/tech/decisions/` con frontmatter conforme a `adr-writing` skill.
- Cada ADR tiene `status: draft`, `created_from: knowledge/tech/discovery/decision-candidates.md`.
- Secciones estándar: Context, Options Considered, Decision, Consequences, Related Capabilities, Related Work Items.
- Donde no hay evidencia suficiente, `Decision` y/o `Consequences` quedan `[open]` para que un humano los cierre.
- `discovery/decision-candidates.md` enlaza a cada ADR materializado.

**Entry points:**
- `knowledge/tech/decisions/ADR-{001..005}-*.md` (nuevos).
- `knowledge/tech/discovery/decision-candidates.md` (actualización).

**End-to-end flow:**
1. Leer `discovery/decision-candidates.md` y `extraction-matrix.md` (ADR-001 depende de la matriz).
2. Materializar cada candidato bloqueante como ADR draft.
3. Marcar `[open]` donde falte evidencia (baseline exacta de Wasmtime en ADR-003, estrategia de memoria concreta en ADR-005).
4. Enlazar desde `decision-candidates.md`.
5. Dejar la aceptación formal (`status: accepted`) como paso humano fuera de este WI.

**Problem:**
INIT-01-01, INIT-02-01/02 e INIT-03-01 no pueden abrirse sin fronteras de arquitectura, independencia del runtime, baseline IR y controles mínimos documentados. Sin ADRs, cada WI reinterpreta la decisión.

**Expected result:**
5 ADR drafts firmables por humano. Base documental para los WIs posteriores.

**Suggested Knowledge Level:** K3

**Impact analysis:**
- Documentation: **affected** — nuevos ADR drafts.
- Content / copy: **affected** — actualización de `decision-candidates.md`.
- Operations / release: **reviewed-not-affected** — las decisiones impactan diseño, no release aún.
- Resto de superficies: not-applicable.

**Module coverage:**
- `knowledge/`: **affected** — nuevos archivos + enlaces.
- Crates planeados: not-applicable (los ADRs los definen; no se crean aquí).

**Scope unknowns:**
- La decisión concreta de ADR-003 sobre la versión exacta de Wasmtime queda `[open]` hasta validación — explícito en el ADR, no un hueco en el WI.
- La estrategia inicial de memoria de ADR-005 queda `[open]` hasta los experimentos de INIT-02-05.

**Scope confidence:** high

Razones:
- Los 5 candidatos ya existen con contexto detallado en `discovery/decision-candidates.md`.
- `AN-002` suministró el insumo técnico para ADR-001 (fronteras de responsabilidades) y ADR-003 (schemas IR v0.5 estables).
- Los `[open]` explícitos acotan lo que no se decide aquí.

## Acceptance criteria

1. **Cinco archivos materializados.** `knowledge/tech/decisions/` contiene ADR-001..005 con frontmatter `type: adr`, `status: draft`, `date: 2026-10-17`, `created_from: knowledge/tech/discovery/decision-candidates.md`.
2. **Secciones estándar.** Cada ADR tiene: Context, Options Considered, Decision, Consequences, Related Capabilities, Related Work Items.
3. **Contexto completo.** Context incluye la fuente del candidato (Open Question, Risk, Business Goal) y la razón por la que la decisión es ahora bloqueante.
4. **Opciones honestas.** Options Considered lista al menos 2 alternativas por ADR, incluyendo la rechazada y su razón.
5. **Decisiones explícitas o `[open]`.** Decision y Consequences son concretas cuando hay evidencia; `[open]` cuando no (ADR-003 Wasmtime version, ADR-005 estrategia memoria concreta).
6. **Trazabilidad.** Cada ADR enlaza a (a) la pieza de la matriz de extracción relevante, (b) las iniciativas que depende de él, (c) riesgos del roadmap (R-xx) que mitiga.
7. **Index actualizado.** `discovery/decision-candidates.md` marca los 5 candidatos como materializados y enlaza a cada ADR por ID.
8. **No se marca `accepted` ninguno.** La aceptación es paso humano fuera de este WI.

## Out of scope

- Marcar los ADRs como `accepted` — es decisión humana explícita por ADR.
- Fijar la versión exacta de Wasmtime en ADR-003 — queda `[open]` con criterio documentado.
- Materializar los ADR diferidos (publicación, firma, resolver, caché, concurrencia) — viven en sus bloques respectivos (RM-04/05/06/08).
- Modificar código de producción (no existe aún).

## Validation

1. Revisión humana ADR por ADR verificando los 8 AC.
2. Comandos:
   ```bash
   ls knowledge/tech/decisions/
   # → ADR-001-*.md, ADR-002-*.md, ..., ADR-005-*.md

   kaddo adr --json | jq '.candidates | length, .materialized | length'
   # → candidates: 5 (los 5 bloqueantes), materialized: 5

   grep -l "^status: draft" knowledge/tech/decisions/ADR-*.md | wc -l
   # → 5

   grep -l "^status: accepted" knowledge/tech/decisions/ADR-*.md | wc -l
   # → 0 (ninguno se auto-acepta)
   ```
3. `kaddo verify AN-003` sobre los AC ejerce los criterios 1–8.

## Definition of Done

- 5 ADR drafts creados conforme a los AC.
- `discovery/decision-candidates.md` actualizado con enlaces.
- Verificación 2 pasa en el host del revisor.
- WI pasa a `completed` tras `kaddo verify`.
- Al menos un seguimiento abierto declarado por cada ADR con `[open]`.

**Open questions:**

- OQ-AN-003-01: ¿la aceptación formal de cada ADR es tarea del humano en el mismo turno, o se abre un WI de revisión por ADR? Propuesta: no abrir WIs; el humano acepta ADR por ADR con un pequeño cambio en el frontmatter (`status: accepted` + `accepted_at`).
- OQ-AN-003-02: ¿se materializan también los ADR candidatos **diferidos** (publicación, firma, resolver, …) como stubs? Propuesta: no — se materializan cuando su bloque los demande; mantener solo bloqueantes aquí.

**Suggested ownership (code globs):**

- `knowledge/tech/decisions/ADR-*.md`
- `knowledge/tech/discovery/decision-candidates.md`

**Related domain / capability:**
- `related_domain`: Foundation & Governance.

**Related decisions:**
- Este WI **crea** las decisiones; no depende de ninguna materializada.

**Related legacy findings:** no aplica.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-01 — Foundation & POC Assessment |
| Roadmap iniciativa | INIT-01-03 — ADRs bloqueantes (001..005) |
| Candidatos consolidados | WI-CANDIDATE-01-03-01..05 |
| Prerrequisitos completados | AN-001 (política `_reference/`), AN-002 (matriz de extracción) |
| Business goals | #1, #4 (separar build vs operación), #5 (ejecución controlada) |
| Principios del roadmap | #3 (runtime no depende LLM), #4 (fail-explicit), #6 (separación responsabilidades) |

## Learning

**Lo que salió como se esperaba:**

- La matriz de extracción (AN-002) alimentó directamente ADR-001 (fronteras de responsabilidades) y ADR-003 (schemas IR v0.5 estables + histórico v2/v3/v4). Hacer AN-002 antes evitó decisiones ciegas.
- Dejar la aceptación formal (`accepted`) como paso humano separado es correcto: los ADRs tienen implicaciones de diseño que merecen revisión explícita.
- `[open]` concretos en ADR-003 (versión Wasmtime) y ADR-005 (allocator alternativo, instancias persistentes, allocación masiva) marcan los seguimientos de forma auditable sin inflar el alcance de los ADRs.

**Lo inesperado:**

- ADR-005 terminó más estructurado de lo previsto: en lugar de un ADR único, estructura **dos regímenes** (baseline operativa pre-M3 + ruta de optimización post-M3 gobernada). Esto refleja mejor la realidad del roadmap sin dividir el ADR.
- ADR-001 cristaliza la condición objetiva (3 criterios alternativos) para extraer `orchestrator` como crate independiente. Esta regla no estaba en `decision-candidates.md` original y surgió al redactar el ADR — es insumo nuevo para RM-04.
- La `publish::` dentro de `cli::` (ADR-001) es más agresiva que lo que decía el roadmap ("capa de orquestación"). Vale la pena revisarla cuando se materialice INIT-04-05.

**Riesgos materializados:** ninguno. Dejar todos los ADRs como `draft` (R-ADR-01) cumplido. R-ADR-02 (fijar baseline sin evidencia) mitigado por `[open]` explícito en ADR-003.

**Deuda y seguimientos:**

- OQ-AN-003-01: resuelta — aceptación por humano con cambio puntual de frontmatter (`status: accepted` + `accepted_at: YYYY-MM-DD`); no se abre WI por ADR.
- OQ-AN-003-02: resuelta — ADR diferidos (publicación, firma, resolver, caché, concurrencia) no se materializan aquí; viven en sus bloques respectivos.
- **Seguimiento transversal:** cuando ADR-001 se acepte, INIT-01-01 puede abrirse con la frontera `publish::` dentro de `cli::` como opción por defecto; cualquier desviación va a un nuevo ADR.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-01-01** (workspace) tiene todo lo que necesita: 4 crates (`ir`, `compiler`, `runtime`, `cli`), reglas de dependencia Cargo auditables por lint, criterios objetivos para evolución futura.
- **INIT-02-01** (`ir`) tiene schema v0.5 como baseline y los 4 schemas históricos como referencia.
- **INIT-02-05** (modelo de memoria) tiene baseline operativa fija (instancias nuevas + copia al host) y lista explícita de `[open]` para experimentar.
- **INIT-03-04/05** (fail-explicit + deny-by-default) tienen ADR-004 como contrato invariante — ningún WI puede introducir fallback silencioso.
- **INIT-05-02** (SDK Rust) tiene prohibida la dependencia transitiva hacia LLM/compiler por ADR-002 + lint CI de INIT-01-01.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-003-adrs-bloqueantes.md
knowledge/tech/decisions/ADR-001..005

Next:
kaddo create --from roadmap (INIT-01-01)
────────────────────────
