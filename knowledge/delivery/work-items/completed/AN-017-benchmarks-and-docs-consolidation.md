---
id: AN-017
type: chore
status: completed
completed_at: 2026-10-18
title: Consolidación de benchmarks y documentación
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
code:
  - crates/runtime/benches/baseline.rs
  - knowledge/tech/baselines/runtime-baseline.md
  - README.md
  - knowledge/tech/current-state.md
  - knowledge/delivery/roadmap.md
source_roadmap_initiative: INIT-03-06
cross_initiatives:
  - INIT-08-01
source_work_item_candidate:
  - Technical Review 2026-10-17 (ref 06d270d) — WI-D
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Performance
  - Distribution & DX
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Las cifras publicadas de rendimiento reflejan lo que realmente miden. README y
  documentos técnicos describen capacidades disponibles sin afirmaciones
  contradictorias. Base limpia para INIT-08-01 (harness formal cross-OS).
risks:
  - >-
    R-D-01: renombrar benches rompe comparaciones históricas con la fila 'AN-012
    original'; mitigado conservando ambos nombres o documentando el mapeo en el
    historial
dependencies:
  - AN-012
  - AN-013
  - AN-014
  - AN-016
source_signals:
  - >-
    Technical Review: cifras corresponden a fixture Core WASM muy pequeño; no
    generalizables
  - >-
    Technical Review: warm_invoke_identity reutiliza componente pero crea nueva
    instancia/Store
  - >-
    Technical Review: cached_cold_start_identity mide cache hit, no cold start
    real
  - 'Technical Review: falta escenario Component Model v0.5'
  - 'Technical Review: README puede quedar fuera de sincronía con el estado real'
  - 'AN-014 cerrado: fixture program-005 (v0.5 Component Model) disponible'
  - >-
    AN-016 cerrado: compile_ir_to_wasm_with_context + CompileContext permiten
    benches reproducibles sin cwd
decision_candidates: []
related_decisions:
  - ADR-003
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-017 — Consolidación de benchmarks y documentación

**Actor y resultado:**
Un lector del README, del roadmap, de `runtime-baseline.md` o del output de `cargo bench` obtiene una descripción consistente y precisa de qué mide cada número y qué funcionalidad está disponible. Las cifras separan por etapa (engine / load / instantiate / invoke) y por tipo de artefacto (Core WASM / Component Model). Las aserciones de mejora no exceden el alcance medido.

**Current behavior:**
- `cold_start_identity`: mide Engine::new + load + invoke, pero usa IR v0.1 → Core WASM, no Component Model. Nombre no indica el tipo.
- `warm_invoke_identity`: nombre sugiere "warm" pero cada iteración crea nuevo Store + Instance (ADR-005 baseline operativa). Lo cacheado es el `wasmtime::Module`, no la instancia.
- `cached_cold_start_identity`: no es un cold start — la caché del Resolver está pre-primed antes del bench. Mide el **cache hit** path, no un cold start.
- `runtime-baseline.md` dice "cold start" sin aclarar que es Core WASM, no Component Model. Las observaciones hacen comparación 186× sobre una base pequeña.
- README menciona "Compilador IR → WASM Component Model determinista con hash SHA-256" — correcto en teoría, pero los benchmarks que lo acompañan solo prueban Core WASM.
- `current-state.md` enlaza al baseline sin matizar el tipo de artefacto medido.
- Memoria no instrumentada (confirmado en AN-012).

**Target behavior:**
- **Renombrar / documentar benches** en `baseline.rs`:
  - `engine_construct_only`: solo `Engine::new()`.
  - `load_core_identity`: `Engine::new()` + `load_component(core_wasm)`.
  - `cold_run_core_identity`: Engine + load + invoke sobre Core WASM.
  - `warm_run_core_identity` (renombrado desde `warm_invoke_identity`): invoca sobre un `Component` ya cargado, aclarando que **cada invocación** crea nuevo Store + Instance (ADR-005).
  - `cache_hit_run_core_identity` (renombrado desde `cached_cold_start_identity`): mide el camino `Resolver::load (hit) + invoke`.
  - **Nuevo:** `cold_run_component_model_point_sum`: equivalente al cold run usando el fixture **v0.5 Component Model `program-005`** entregado por AN-014. Usa `compile_ir_to_wasm_with_context` con un `CompileContext` apuntando al compiler crate (AN-016). Invoca `"an:test/ops@0.1.0#make-point-sum"`.
  - **Nuevo:** `cache_hit_run_component_model_point_sum`: contraparte Component Model usando `Resolver`.
  - Aclaración documental: la ausencia de un bench para "record anidado" NO es un gap; AN-016 documentó que IR v0.5 rechaza record-dentro-de-record por diseño, así que no hay escenario que medir hasta IR v0.6+.
- **`runtime-baseline.md` reescrito** con:
  - Tabla separada por **tipo de artefacto** (Core / Component Model).
  - Columna explícita "etapa medida" (Engine / Load / Instantiate / Invoke).
  - Columna "hardware", "toolchain", "método" (criterion `--quick` vs completo).
  - Sección "qué NO mide esta tabla" (cross-OS, memoria, concurrencia, producción).
  - Historial preservado con mapeo de nombres antiguos → nuevos.
- **README actualizado:**
  - Sección "Estado del proyecto" revisada para no afirmar capacidades no probadas E2E (coherente con AN-014).
  - Sección de rendimiento, si existe, con disclaimers idénticos a `runtime-baseline.md`.
- **current-state.md** y **roadmap.md** alineados: el baseline link indica tipo medido; AN-014 ya cerrado → menciona la cobertura E2E v0.5 lograda, en lugar de "pendiente".
- **Instrucciones de reproducción** compiladas en una sección del baseline doc, exactas y copiables.

**Problem:**
Cifras y nombres actuales inducen a conclusiones falsas: lector asume "runtime es rápido con componentes WIT" cuando solo probamos Core WASM identity. README afirma capacidades (Component Model determinista) sin benchmarks que las acompañen. Deuda documental que crece a cada WI nuevo.

**Scope confidence:** high. Es trabajo mecánico de renombrar + reescribir docs + añadir 2 escenarios Component Model (si AN-014 provee fixture).

## Acceptance criteria

1. **Renombrados aplicados** en `baseline.rs` según target behavior.
2. **Historial del baseline doc** preserva el mapeo antiguo → nuevo (no borra datos previos).
3. **Tabla segregada** por tipo (Core / Component Model) y etapa, usando las etapas realmente medidas por la suite: `engine_construct`, `load`, `cold_run`, `warm_run`, `cache_hit_run`. La **instanciación como etapa aislada no se mide** por este WI (requeriría un bench adicional específico y un split interno de `invoke` que excede el alcance; INIT-08-01 puede añadirlo).
4. **Cada número publicado** especifica hardware + toolchain + método + alcance.
5. **Afirmaciones de mejora acotadas** al tipo y etapa medidos (p. ej. "cache hit vs cold run, Core WASM, Windows local").
6. **README y roadmap y current-state** no se contradicen entre sí sobre lo que está disponible E2E.
7. **Instrucciones de reproducción** copiables y verificadas desde el baseline doc.
8. **AN-014/015/016 ya cerrados:** benches Component Model activos con cifras reales (sin `#[ignore]`). Usan `program-005` y `compile_ir_to_wasm_with_context`.
9. **Tests existentes (43) siguen verdes.** CI remota verde 3 OS.

## Out of scope

- Optimizaciones adicionales (fuera de AN-013).
- Benchmarks cross-OS matrix (INIT-08-01).
- Compromisos de rendimiento en producción (RM-08).
- Instrumentación de memoria con `ResourceLimiter` (INIT-06-02 / INIT-08-02).
- Concurrencia (RM-08).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p agent-native-runtime --bench baseline -- --quick

# Linter de coherencia manual:
diff <(grep -oE "funcional[a-z]*" README.md | sort -u) \
     <(grep -oE "funcional[a-z]*" knowledge/tech/current-state.md | sort -u)
```

## Definition of Done

- Benches renombrados y descritos.
- `runtime-baseline.md` reescrito con tabla segregada + disclaimers.
- README y current-state revisados; sin contradicciones.
- CI remota verde.
- WI `completed`.

**Open questions:**

- OQ-AN-017-01: ¿los nombres antiguos de benches se preservan como alias temporales en `baseline.rs` o se eliminan directamente? Propuesta: eliminar y mantener historial solo en el doc, para evitar bloat en `baseline.rs`.
- OQ-AN-017-02: Technical Review menciona "métricas de memoria y número de instancias cuando exista instrumentación adecuada" — propuesta: dejar como sección "no medido todavía" con enlace a INIT-06-02; AN-017 no instrumenta por sí mismo.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 + RM-08 |
| Roadmap iniciativa | INIT-03-06 (principal), INIT-08-01 (prepara) |
| ADRs | ADR-003 |
| Technical Review | WI-D (ref 06d270d) |
| Prerrequisitos | AN-012, AN-013, AN-014, AN-016 |

## Learning

**Lo que salió como se esperaba:**

- Los 9 benches corren limpios bajo `criterion --quick`; medianas estables en 2-3 corridas consecutivas.
- La convención `<stage>_<path>_<fixture>` deja los nombres autodescriptivos sin necesidad de leer el código.
- Reutilizar `program-005` + `compile_ir_to_wasm_with_context` como fixture Component Model funciona sin fricción; AN-014/AN-016 habían dejado todo listo.
- La tabla segregada por path × etapa resuelve el problema del Technical Review: cada fila es comparable por sí sola, sin inducir conclusiones agregadas falsas.

**Lo inesperado:**

- **Clippy pedantic `doc_markdown`** se queja de los nombres técnicos en el header del bench file (p. ej. `wasm_component_model`). Suprimido a nivel de archivo con `#![allow(clippy::doc_markdown)]` dado que es doc narrativa, no API pública.
- Las medianas AN-017 variaron ±5-10% vs AN-012/013 (p. ej. `load_only` 247 µs → `load_core_identity` 229 µs; `cached_cold_start` 2.09 µs → `cache_hit_run_core` 1.89 µs). Es ruido esperado de `--quick` en workstation personal; documentado en el historial del baseline doc.
- `cold_run_component_model_point_sum` (~601 µs) es ~2.6× más caro que `cold_run_core_identity` (~230 µs). El gap refleja parsing + bindings WIT del Component Model, no una regresión; documentado en Observaciones del baseline.

**Riesgos materializados:** ninguno. R-D-01 (romper comparaciones históricas) mitigado — el mapa old→new preservado en el historial permite leer commits anteriores.

**Deuda y seguimientos:**

- **ADR-003 L145** (versión exacta de Wasmtime baseline) sigue abierta; ADR-003 lo deferea con criterio objetivo.
- **Memoria por instancia:** diferida a INIT-06-02 (ResourceLimiter).
- **Cross-OS:** diferido a INIT-08-01.
- **Comparación con nativo:** diferida a INIT-08-02 con metodología justa.
- **Record anidado como escenario de bench:** no aplica — IR v0.5 lo rechaza por diseño (AN-016 Learning). Re-abrir cuando IR v0.6+ relaje la restricción.

**Lo que esta iniciativa establece:**

- **El Technical Review de 2026-10-17 queda cerrado al 100%:** AN-014 + AN-015 + AN-016 + AN-017 cumplieron los 4 WIs propuestos (WI-A/B/C/D).
- **INIT-08-01** (harness formal cross-OS) tiene base clara: reutilizar `baseline.rs` + añadir matrix. Los 9 benches ya nombrados correctamente no requerirán renombrados futuros.
- **INIT-05-01** (CLI) puede exponer flags que mapean 1:1 a los benches (`--benchmark=cold_run_core_identity`, etc.) sin crear nueva nomenclatura.
- **INIT-06-02** (fuel + instance limits) tiene criterio objetivo cuantificado para evaluar su impacto: cualquier instrumentación adicional debe cuantificar su overhead en estas 9 filas.

### Evidencias de ejecución

- Commit implementación: `39c2182` → CI remota [#39147188886](https://github.com/Kaddo-kdd/agent-native/actions/runs/39147188886) **success** 3/3 OS.
- Commit cierre: `90b3eb0` → CI remota [#39147372753](https://github.com/Kaddo-kdd/agent-native/actions/runs/39147372753) **success** 3/3 OS.
- Local Windows: `cargo test --workspace` → 43 pruebas, 0 fallos.
- Local Windows: `cargo bench -p agent-native-runtime --bench baseline --no-run` → **Finished `bench` profile [optimized]**, ejecutable `baseline-d679b4e2cfbb2ad1.exe` producido. Confirma que los 9 benches compilan bajo perfil release sin warnings bajo `clippy pedantic -D warnings` (incluido en el workflow CI).
- `kaddo verify AN-017 --json`: `"readiness": "READY_TO_COMPLETE"`, 0 blockers, 0 warnings, 0 AC not-verified.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-017-benchmarks-and-docs-consolidation.md
crates/runtime/benches/baseline.rs (reescrito)
knowledge/tech/baselines/runtime-baseline.md (reescrito)
knowledge/tech/current-state.md (actualizado)
knowledge/delivery/roadmap.md (INIT-08-01 referenciado)

Next: Technical Review 2026-10-17 cerrado. Siguiente iniciativa del
roadmap: INIT-03-03 (executor YAML) para alcanzar M3 — Runtime
autónomo con regresión POC completa.
────────────────────────
