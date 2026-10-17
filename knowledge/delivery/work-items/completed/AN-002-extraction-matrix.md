---
id: AN-002
type: spike
status: completed
completed_at: 2026-10-17
title: Matriz de extracción POC → productos
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - knowledge/tech/extraction-matrix.md
source_roadmap_initiative: INIT-01-02
source_work_item_candidate:
  - WI-CANDIDATE-01-02-01
  - WI-CANDIDATE-01-02-02
related_domain: Foundation & Governance
domains:
  - Foundation & Governance
related_capabilities: []
expected_value: >-
  Mapa decisorio pieza-a-pieza de la POC para ordenar la extracción en
  RM-02/03/04 y evitar reutilización ciega de código experimental.
risks:
  - 'R-POC-01: sobrestimar reutilización y heredar deuda experimental'
  - 'R-POC-02: subestimar reutilización y rehacer trabajo validado'
dependencies: []
source_signals:
  - 'Open Question L154: qué artefactos POC se migran como fuente trazable'
  - 'Roadmap Candidate Signal: ordenar RM-02/03/04 depende de este mapa'
decision_candidates: []
related_decisions: []
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-17'
---

# AN-002 — Matriz de extracción POC → productos

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

**Actor and outcome:**
Equipos de ingeniería y agentes disponen de una matriz canónica que, por cada pieza de la POC `_reference/poc-agent-native/@26b6a18`, declara una decisión (`migrate | redesign | drop`), un destino en el producto nuevo, pruebas asociadas y responsable candidato. La matriz ordena RM-02/03/04 y evita reutilización ciega.

**Current behavior:**
`knowledge/tech/codebase.md:55` tiene una tabla preliminar con 7 filas de alto nivel. No cubre todas las piezas (fixtures WIT, suites YAML, specs narrativas, schemas IR, reports de benchmark, runs E2E) y no asigna responsable ni pruebas por fila.

**Target behavior:**
`knowledge/tech/extraction-matrix.md` canónico con:
- Fila por pieza observable de la POC (crates, módulos, fixtures WIT, suites YAML, schemas, specs, runs).
- Decisión por fila (`migrate | redesign | drop`) justificada.
- Destino conceptual en el producto nuevo (crate, carpeta o "no destino").
- Pruebas asociadas (o justificación de ausencia).
- Responsable candidato (equipo o rol).
- Riesgos de la migración por fila cuando aplique.

**Entry points:**
- `knowledge/tech/extraction-matrix.md` (nuevo).
- Enlace desde `knowledge/tech/codebase.md` (sección "Mapa de extracción desde la POC") a la matriz canónica.

**End-to-end flow:**
1. Agente/humano explora `_reference/poc-agent-native/@26b6a18` respetando `reference-policy.md` (consulta autorizada, implementación fuera de `_reference/`).
2. Cataloga cada pieza observable con metadatos: ruta POC, tipo, LOC, pruebas asociadas, dependencias.
3. Para cada pieza propone decisión y destino conceptual con justificación.
4. Redacta `knowledge/tech/extraction-matrix.md` siguiendo el formato acordado.
5. Enlaza desde `codebase.md`.

**Problem:**
Sin matriz exhaustiva, cada WI posterior reinterpreta qué de la POC es relevante. Deriva en reutilización ciega o descarte accidental de evidencia validada.

**Expected result:**
Documento canónico que ordena RM-02/03/04 y permite abrir `kaddo create --from roadmap` para INIT-01-03/01-01 y los bloques siguientes sin ambigüedad sobre qué migrar.

**Suggested Knowledge Level:** K3

**Impact analysis:**
- Product / UI: not-applicable
- Frontend: not-applicable
- Backend: not-applicable (no hay backend aún)
- Database: not-applicable
- Configuration: not-applicable
- Content / copy: **affected** — nuevo `extraction-matrix.md`; actualización menor en `codebase.md`
- Authentication / authorization: not-applicable
- Notifications: not-applicable
- Analytics: not-applicable
- Documentation: **affected** — matriz canónica
- Operations / release: **reviewed-not-affected**

**Module coverage:**
- `crates/ir` (planeado): not-applicable (no se crea aquí; se decide qué migrar hacia él).
- `crates/compiler` (planeado): not-applicable (ídem).
- `crates/runtime` (planeado): not-applicable (ídem).
- `crates/cli` (planeado): not-applicable.
- `knowledge/`: **affected** — nuevo archivo + enlace.
- `_reference/poc-agent-native/`: **reviewed-not-affected** — solo lectura (política AN-001).

**Scope unknowns:**
- Si durante la catalogación se detecta una pieza nueva (fixture oculto, reporte adicional, variante de dominio) se añade a la matriz. No se re-abre decisión fuera de alcance.
- La elección de "responsable candidato" por fila es orientativa (K3), no vinculante.

**Scope confidence:** high

Razones:
- POC congelada en commit `26b6a18` → superficie estable.
- No introduce código ejecutable.
- Verificación consiste en revisión humana + presencia de secciones obligatorias.

## Acceptance criteria

1. **Cobertura completa.** La matriz lista, al menos: los 3 crates principales (`irtool`, `orchestrator`, `runner`), los 4 crates baseline-rust, los 9 contratos WIT, las ~19 suites YAML de `tests/`, los schemas IR (`ir.schema*.json`), los specs narrativos de `spec/`, los runs E2E (`runs/*`, especialmente `unknown-domain-e2e`), y los 5 reportes de benchmark + release candidate.
2. **Decisión por fila.** Cada fila tiene `migrate | redesign | drop` con una justificación de una línea.
3. **Destino.** Cada fila `migrate` o `redesign` nombra un destino conceptual (crate, carpeta o módulo) existente en el roadmap.
4. **Pruebas asociadas.** Cada fila `migrate` referencia las pruebas (unitarias/aceptación) que validan la migración, o declara explícitamente "sin pruebas asociadas en POC" cuando corresponde.
5. **Responsable candidato.** Cada fila nombra un responsable candidato (equipo o rol: IR, compiler, runtime, orquestación, QA, release).
6. **Riesgos.** Toda fila `migrate | redesign` cuyo riesgo no sea trivial declara el riesgo específico o enlaza a un `R-xx` del roadmap.
7. **Trazabilidad.** La matriz enlaza desde `knowledge/tech/codebase.md`.
8. **Totales declarados.** El pie de la matriz reporta totales por decisión (N migrate / N redesign / N drop) y comenta cualquier concentración relevante.

## Out of scope

- Materializar los Work Items derivados (van a INIT-02-xx, INIT-03-xx, INIT-04-xx).
- Clonar/sincronizar la POC (ya cubierto por AN-001).
- Redactar ADRs (van a INIT-01-03).
- Decidir qué contenido migrar bit a bit (la matriz decide la pieza, el detalle vive en los WIs).

## Validation

1. Revisión humana del documento `knowledge/tech/extraction-matrix.md` verificando los 8 criterios de aceptación.
2. Grep de piezas de la POC vs filas de la matriz para confirmar cobertura:
   ```bash
   # LOC por crate en la POC (referencia de completitud)
   wc -l _reference/poc-agent-native/crates/*/src/*.rs
   # Fixtures WIT
   ls _reference/poc-agent-native/wit/
   # Suites YAML
   ls _reference/poc-agent-native/tests/
   # Specs narrativos
   ls _reference/poc-agent-native/spec/
   # Runs E2E
   ls _reference/poc-agent-native/runs/ | head -40
   ```
   Cada archivo/directorio devuelto debe aparecer en la matriz o estar justificado en su exclusión agregada.
3. `kaddo guard` sobre el cambio no debe reportar deriva en artefactos no relacionados.
4. `kaddo verify AN-002` (una vez `in-progress`) ejerce los AC sobre la evidencia registrada.

## Definition of Done

- `knowledge/tech/extraction-matrix.md` creado y cumple los 8 criterios de aceptación.
- `knowledge/tech/codebase.md` enlaza a la matriz.
- La verificación 2 produce cobertura completa en el host del revisor.
- El WI pasa a `completed` tras `kaddo verify`.

**Open questions:**

- OQ-AN-002-01: ¿la matriz incluye los directorios `build/`, `knowledge/` y `.github/` de la POC o se marcan como "no destino" agregado? Propuesta: agregado explícito como "no destino, usar política de referencia".
- OQ-AN-002-02: ¿los 4 `baseline-rust*` se tratan individualmente o agregados? Propuesta: fila agregada (`drop`), ya que no son parte del producto sino comparativos.

**Suggested ownership (code globs):**

- `knowledge/tech/extraction-matrix.md`
- `knowledge/tech/codebase.md`

**Related domain / capability:**
- `related_domain`: Foundation & Governance (candidato de dominio).

**Related decisions:**
- `decision_candidates`: ADR-001 (arquitectura inicial) depende de la decisión por crate que proponga esta matriz, pero no la bloquea — ADR-001 se materializa en INIT-01-03 **después** de esta matriz.

**Related legacy findings:** no aplica — la POC no es "legacy".

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-01 — Foundation & POC Assessment |
| Roadmap iniciativa | INIT-01-02 — Matriz de extracción POC → productos |
| Candidatos consolidados | WI-CANDIDATE-01-02-01, WI-CANDIDATE-01-02-02 |
| Business goals | #1 (reducir fricción), #7 (decisiones basadas en evidencia) |
| Principio del roadmap | #1 (Recuperar antes de innovar) |
| Prerrequisito completado | AN-001 (política `_reference/`) |

## Learning

**Lo que salió como se esperaba:**

- La POC tiene superficie estable (3 crates principales, 9 WIT, 18 suites YAML, 35 runs, 2 reportes clave) y la catalogación cabe en una sola pasada.
- Agrupar schemas y programas IR en 2 filas (S-01, S-02) en lugar de una por archivo mantiene la matriz legible sin perder cobertura.
- Separar `irtool` en `ir` (modelo + verifier) y `compiler` (backend + hash) es la decisión más limpia: `irtool/lib.rs` ya tiene la frontera marcada internamente por `pub fn verify_ir` vs `pub fn compile_ir_to_wasm`.

**Lo inesperado:**

- `crates/orchestrator/src/lib.rs` concentra **tres** responsabilidades (SystemSpec, build end-to-end y publicación atómica). La POC las mezcla sin fronteras internas visibles — rediseñar la publicación como módulo aislado requiere lectura detallada antes de abrir INIT-04-05.
- Hay 35 directorios `runs/`, mucho más de lo esperado. La mayoría son evidencia histórica por iteración (`poc002-*`..`poc005-*`); solo 2 (`unknown-domain-e2e`, `standalone-validation`) son valor migrable; el resto se queda en `_reference/`.
- El `SynthesizerAgent` de la POC está acoplado a dominios específicos (`synthesize_shipping_ir`, `synthesize_order_ir`, …). Rediseñarlo como "sintetizador determinista genérico" en INIT-04-02 implica más trabajo del que sugería `codebase.md`.

**Riesgos materializados:** ninguno. R-POC-01 (sobrestimar reutilización) y R-POC-02 (subestimar) quedan declarados por fila en la matriz; se revisarán al abrir los WIs derivados.

**Deuda y seguimientos:**

- OQ-AN-002-01 (directorios agregados `build/`, `knowledge/`, `.github/`): resueltos como `drop` agregado en la sección 9.
- OQ-AN-002-02 (baseline-rust): resuelto como `drop` agregado (fila C-08) en la sección 2.
- **Nueva señal para ADR-001 (INIT-01-03):** el límite entre "orchestrator como crate" y "orquestación dentro del bin cli" debe elegirse considerando la triple responsabilidad de `orchestrator/lib.rs`. Propuesta mínima: módulos internos con fronteras estrictas (`spec::`, `build::`, `publish::`), sin crate adicional en MVP.
- La reutilización de los `.wit` como **fixtures** (sección 3) exige que INIT-02-03 añada un lint CI que impida colocar un `.wit` de dominio en `wit/public/`.

**Lo que esta iniciativa establece para las siguientes:**

- INIT-01-03 (ADRs) tiene insumo concreto para redactar ADR-001 (arquitectura inicial con fronteras internas) y ADR-003 (schema IR v0.5 como baseline + política de evolución v0.6).
- INIT-01-01 (workspace) puede materializarse con los crates `ir`, `compiler`, `runtime` y bin `cli` sin ambigüedad sobre qué contenido técnico recibe cada uno.
- INIT-02-01/02 pueden abrirse en paralelo: `ir` consume C-01 (verifier/model) + S-01 + S-02 + N-01; `compiler` consume C-01 (backend) sin dependencia hacia `orchestrator`.
- INIT-04-05 (publicación atómica) tiene identificado desde el inicio que es un **módulo** de la capa de orquestación, no del `compiler`.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-002-extraction-matrix.md
knowledge/tech/extraction-matrix.md

Next:
kaddo create --from roadmap (INIT-01-03)
────────────────────────
