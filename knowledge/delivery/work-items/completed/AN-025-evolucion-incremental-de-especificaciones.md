---
id: AN-025
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-025 — Evolución incremental de especificaciones y reconciliador SystemSpec"
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/cli/src/spec/diff.rs
  - crates/cli/src/spec/reconciler.rs
  - crates/cli/src/spec/mod.rs
  - crates/cli/src/spec/error.rs
  - crates/cli/tests/spec_evolution.rs
source_roadmap_initiative: INIT-04-03
initiative: INI-004
domains:
  - Specification & Knowledge Ingestion
  - Validation & Evolution
related_capabilities:
  - Versioned Publication & Evolution
expected_value: >-
  Permitir la evolución incremental acumulativa de especificaciones (v_n -> v_{n+1}),
  reconciliando nuevos Work Items o deltas funcionales con un SystemSpec base,
  calculando el diff semántico detallado (SpecDiff) y detectando colisiones o regresiones
  antes de la fase de compilación y publicación atómica.
dependencies:
  - AN-023
  - AN-024
source_signals:
  - "POC crates/orchestrator/src/lib.rs (parse_work_items_consolidated, compute_spec_diff)"
  - "Roadmap RM-04: INIT-04-03"
  - "Principio de Comportamiento Acumulativo"
  - "ADR-001: Arquitectura inicial del workspace"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-025 — Evolución incremental de especificaciones y reconciliador SystemSpec

**Actor y resultado:**
Un desarrollador, orquestador o agente proporciona un `SystemSpec` base (versión $v_n$) y uno o más deltas de requisitos o Work Items en formato estructurado (Markdown / frontmatter). El reconciliador aplica las modificaciones produciendo un nuevo `SystemSpec` evolucionado ($v_{n+1}$) con versionamiento monotónico, reporta un `SpecDiff` exhaustivo (requisitos añadidos, modificados, obsoletos, componentes afectados y contratos modificados), y previene conflictos o versiones regresivas de forma fail-explicit.

**Current behavior:**
- AN-023 proveyó el parser inicial de narrativa Markdown a `SystemSpec` (`MarkdownRequirementsInterpreter`).
- AN-024 proveyó el contrato `LlmProvider` y los modos de generación (`real`, `fixtures`, `synthesizer`).
- No existía un mecanismo de reconciliación incremental para evolucionar un `SystemSpec` acumulativamente sin reparsear desde cero, ni cálculo de diffs semánticos (`SpecDiff`) para auditar qué componentes y contratos fueron impactados por un cambio.

**Target behavior:**
- Implementar en `crates/cli/src/spec/`:
  - `diff.rs`: Estructura `SpecDiff` y función `compute_spec_diff(before: &SystemSpec, after: &SystemSpec) -> SpecDiff`.
  - `reconciler.rs`: Función `reconcile_spec(base: &SystemSpec, updates: &[(&str, &str)]) -> Result<SystemSpec, SpecError>` que parsea Work Items / deltas de especificación, valida que las actualizaciones de versión sean estrictamente monotónicas ($v_{req} > v_{base}$ o reemplazos válidos), preserva requisitos no modificados y asocia nuevas reglas a las operaciones del componente correspondiente.
  - Actualización de `crates/cli/src/spec/error.rs` con variantes tipadas para conflictos de reconciliación y referencias rotas (`ReconciliationConflict`, `BrokenReference`).
- Pruebas de integración offline en `crates/cli/tests/spec_evolution.rs`:
  - Evolución incremental de $v_1 \to v_2 \to v_3$.
  - Detección de colisiones de versión o intentos de degradación de versiones de requisitos.
  - Referencias rotas ante Work Items que declaran reemplazar requisitos inexistentes.
  - Cálculo exacto de `SpecDiff` auditando componentes afectados y modificaciones de contratos.

## Acceptance criteria

### AC-01 — Estructura y Cálculo de `SpecDiff`
- `SpecDiff` cuantifica:
  - `added_requirements: Vec<String>`
  - `modified_requirements: Vec<String>`
  - `deprecated_requirements: Vec<String>`
  - `affected_components: Vec<String>`
  - `modified_operations: Vec<String>`
  - `modified_entities: Vec<String>`
  - `modified_contracts: Vec<String>`
- `compute_spec_diff` detecta de forma determinista y exhaustiva las diferencias entre dos especificaciones `before` y `after`.

### AC-02 — Reconciliación Acumulativa Monotónica
- `reconcile_spec` actualiza un `SystemSpec` base aplicando una lista de deltas de requisitos.
- Si un Work Item actualiza un requisito existente con una versión menor o igual a la previa con una declaración distinta, falla explícitamente (`SpecError::ReconciliationConflict`).
- Si un Work Item declara `replaces: TARGET` pero `TARGET` no existe en la especificación base, falla explícitamente (`SpecError::BrokenReference`).

### AC-03 — Preservación del Comportamiento Acumulativo
- Los requisitos, componentes, entidades y contratos no modificados en el delta se preservan intactos en la especificación resultante $v_{n+1}$.
- Los nuevos requisitos añadidos se asocian a las operaciones de su componente para mantener la trazabilidad completa.

### AC-04 — Cobertura de Pruebas Offline
- Pruebas exhaustivas en `crates/cli/tests/spec_evolution.rs` sin dependencias de red ni variables de entorno.
- Compatibilidad comprobada con el modelo canónico `SystemSpec` y el validador de Semantic IR v0.5.

## Out of scope
- Publicación atómica filesystem y manifiestos de versiones en disco (corresponde a INIT-04-05).
- Trazabilidad y metadatos de commits hacia Kaddo (corresponde a INIT-04-04).

## Validation
```bash
cargo test -p agent-native
cargo test --workspace
cargo clippy --all-targets --workspace
```

## Traceability

| Elemento | Ubicación |
|---|---|
| Modelo y cálculo de diferencias | `crates/cli/src/spec/diff.rs` |
| Reconciliador de especificaciones | `crates/cli/src/spec/reconciler.rs` |
| Errores de reconciliación | `crates/cli/src/spec/error.rs` |
| Exportación pública en biblioteca | `crates/cli/src/spec/mod.rs` |
| Pruebas de integración offline | `crates/cli/tests/spec_evolution.rs` |

## Learning

**Lo que salió como se esperaba:**
- Reconciliación incremental fiel a la POC y a ADR-001: las funciones residen dentro del módulo `spec::` de `agent_native` sin fragmentar innecesariamente el workspace.
- Versionamiento monotónico estricto: cualquier intento de actualizar un requisito existente con versión idéntica o inferior y contenido discrepante genera un error explícito `ReconciliationConflict`.
- Comportamiento acumulativo: los requisitos no afectados se preservan intactos, los nuevos se incorporan a las operaciones correspondientes y los deltas se auditan con granularidad mediante `SpecDiff`.
- La suite de pruebas de evolución valida la cadena completa de evolución $v_1 \to v_2 \to v_3$ partiendo de narrativas Markdown y reconciliando deltas sucesivos.

**Lo inesperado:**
- El parser de deltas de requisitos es ligero y no requiere una dependencia pesada hacia un parser YAML completo para procesar los frontmatters de los Work Items, preservando los tiempos de compilación mínimos del workspace.
- La suite del workspace sumó 6 nuevas pruebas de integración, totalizando 93 pruebas en el workspace principal y 115 pruebas globales con los ejemplos de AWS Lambda.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --all-targets --workspace`: 0 warnings.
   - `cargo fmt -- --check`: Formato estricto cumplido al 100%.
2. **Pruebas de la suite `agent-native`:**
   - 3 pruebas unitarias spec.
   - 4 pruebas de integración spec interpreter.
   - 5 pruebas de integración agent providers.
   - 6 pruebas de integración spec evolution.
   - Total `agent-native`: 18 passed; 0 failed.
3. **Workspace y Ejemplos:**
   - 93 pruebas del workspace principal superadas.
   - 22 pruebas de AWS Lambda superadas.
   - Total global: 115 passed; 0 failed.
