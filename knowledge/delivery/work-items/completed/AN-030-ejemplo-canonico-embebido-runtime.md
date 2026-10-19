---
id: AN-030
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-030 — Ejemplo canónico de embebido externo del runtime"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/examples/embed_runtime.rs
  - crates/runtime/tests/embed_example_regression.rs
  - examples/README.md
source_roadmap_initiative: INIT-05-02
initiative: INI-005
domains:
  - Runtime & Memory
  - Distribution & DX
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Proveer un ejemplo canónico, completamente documentado y reproducible (`crates/runtime/examples/embed_runtime.rs`)
  que demuestre cómo un consumidor externo de Rust puede embeber `agent-native-runtime` como dependencia,
  configurar políticas de seguridad deny-by-default, cargar artefactos WebAssembly (Core y Component Model),
  invocar operaciones con argumentos tipados `Val`, gestionar errores tipados `RuntimeError` y garantizar
  aislamiento de sesiones mediante `Resolver::reset()`.
dependencies:
  - AN-029
source_signals:
  - "Roadmap RM-05: INIT-05-02 (WI-CANDIDATE-05-02-02)"
  - "INI-005 Criterio 4: Ejemplo embed_runtime.rs compila y ejecuta un componente publicado desde Rust externo"
  - "ADR-001, ADR-002, ADR-005: Runtime autónomo, deny-by-default y aislamiento de sesiones"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T17:50:00.000Z
ready_at: '2026-10-19'
---

# AN-030 — Ejemplo canónico de embebido externo del runtime

**Actor y resultado:**
Un desarrollador Rust externo que desea integrar la ejecución de componentes `agent-native` en su propia aplicación (servidor web, CLI de terceros, servicio en la nube o worker asíncrono) dispone de un ejemplo canónico de referencia (`crates/runtime/examples/embed_runtime.rs`) ejecutable vía `cargo run --example embed_runtime`, con código idiomático, comentarios arquitectónicos claros y sin dependencias no declaradas.

**Acceptance Criteria Verificados:**
- **AC-01 (Ejemplo ejecutable y canónico):** Implementado en [`crates/runtime/examples/embed_runtime.rs`]. Ejecutable sin argumentos (sintetiza el fixture canónico Component Model `program-005`) o con una ruta externa (`cargo run --example embed_runtime -- path/to/component.wasm`).
- **AC-02 (Demostración integral de la API del Runtime):**
  1. Instanciación del motor (`Engine::new()`).
  2. Carga y verificación de componente (`Engine::load_component_with_policy`).
  3. Configuración declarativa de políticas de seguridad (`HostPolicy::deny_all()` y autorización granular de interfaces WIT).
  4. Invocación de operaciones exportadas (`double-value` pasando `Val::S32(21)` -> `42`, y `make-point-sum` retornando `Val::Record`).
  5. Captura y manejo estructurado de errores con la taxonomía `RuntimeError::FunctionNotFound`.
  6. Demostración de reutilización con caché content-addressed y limpieza de sesión mediante `Resolver::reset()`.
- **AC-03 (Independencia de Fronteras ADR-001/ADR-002):** El runtime mantiene cero dependencias del compilador o LLMs en tiempo de ejecución.
- **AC-04 (Prueba de Regresión Automatizada en CI):** Implementada en [`crates/runtime/tests/embed_example_regression.rs`] ejecutando el binario del ejemplo en un subproceso y verificando salida y código 0.

**Evidencia de Verificación:**
- `cargo run --example embed_runtime`: completado con éxito (código 0).
- `cargo test --test embed_example_regression`: `1 passed; 0 failed`.
- `cargo test --workspace`: `125 passed; 0 failed`.
- `cargo clippy --all-targets --workspace`: 0 advertencias.
- `cargo fmt --check`: limpio.
