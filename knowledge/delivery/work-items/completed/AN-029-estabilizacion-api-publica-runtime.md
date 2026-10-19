---
id: AN-029
type: chore
status: completed
completed_at: 2026-10-19
title: "AN-029 — Estabilización y curación de la API pública existente del runtime"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/lib.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/policy.rs
  - crates/runtime/src/resolver.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/tests/public_api_audit.rs
source_roadmap_initiative: INIT-05-02
initiative: INI-005
domains:
  - Runtime & Memory
  - Distribution & DX
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Consolidar y documentar la API pública existente de `agent-native-runtime` (`Engine`, `Component`,
  `HostPolicy`, `RuntimeError`, `Val`, `Resolver`) sin inventar abstracciones artificiales, garantizando
  documentación completa en rustdoc, verificación de aislamiento de tipos privados y pruebas
  explícitas de consumo de la API pública desde crates externos.
dependencies:
  - AN-028
source_signals:
  - "Roadmap RM-05: INIT-05-02"
  - "Decisión Humana D5 (2026-10-19): No crear tipos abstractos artificiales; evaluar primero las interfaces existentes"
  - "ADR-001 y ADR-002: Runtime autónomo e independiente de compiler y LLM"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T17:40:00.000Z
ready_at: '2026-10-19'
---

# AN-029 — Estabilización y curación de la API pública existente del runtime

**Actor y resultado:**
Un desarrollador Rust externo que integra `agent-native-runtime` como dependencia en su aplicación (CLI, servicio HTTP, lambda o worker) puede instanciar el motor (`Engine`), cargar artefactos WebAssembly Component Model o Core WASM (`Component`), aplicar políticas de seguridad declarativas deny-by-default (`HostPolicy`), ejecutar funciones exportadas mediante tipos de Component Model (`Val`), gestionar fallos mediante la taxonomía tipada (`RuntimeError`) y optimizar cargas repetidas mediante resolución content-addressed (`Resolver`), con documentación exhaustiva en rustdoc y sin fuga de tipos internos.

**Acceptance Criteria:**
- **AC-01 (Sin Abstracciones Artificiales - Decisión D5):** La API pública se fundamenta estrictamente en los tipos concretos verificados:
  - `Engine`: constructor y cargador de componentes con o sin política (`new`, `load_component`, `load_component_with_policy`). Implementa `Debug` y `Clone`.
  - `Component`: invocación tipada de funciones (`invoke`, `invoke_core_with_host_ctx`, `invoke_component_with_host_ctx`), inspección de formato (`is_core_module`, `is_component_model`) y acceso a su política (`policy()`). Implementa `Debug`.
  - `HostPolicy`: constructor `deny_all()`, métodos de autorización granular (`allow_core`, `allow_wit_interface`, `allow_wit_operation`, `allow_module`) y consultas de estado (`is_allowed_core`, `is_wit_interface_allowed`, `is_wit_operation_allowed`).
  - `RuntimeError`: taxonomía de errores tipados sin códigos mágicos ni fallbacks silenciosos per ADR-004.
  - `Val`: re-export ergonómico de `wasmtime::component::Val`.
  - `Resolver`: cargador con caché content-addressed por hash SHA-256 (`load`, `load_with_policy`, `cached_count`) y método `reset()` para aislamiento de sesiones. Implementa `Debug`.
- **AC-02 (Curación de Rustdoc y Ejemplos de Uso):** La raíz del crate [`crates/runtime/src/lib.rs`] cuenta con documentación integral de invariantes arquitectónicos (ADR-001/002/005), guía de la API pública y doctest funcional.
- **AC-03 (Encapsulamiento e Invariantes ADR-001/ADR-002):** Ningún tipo interno (`Payload`, `CachedPayload`, parsers internos o submódulos privados) se expone accidentalmente en la superficie pública. La regla `cargo tree -p agent-native-runtime --edges normal` se mantiene estrictamente aislada (solo `agent-native-ir`, `wasmtime`, `serde`, `thiserror`, `sha2`).
- **AC-04 (Prueba de Integración de API Pública Externa):** Suite de prueba dedicada ([`crates/runtime/tests/public_api_audit.rs`]) que importa y utiliza exclusivamente los símbolos re-exportados en la raíz de `agent_native_runtime` para verificar la ergonomía, completitud del API e invariantes de seguridad desde la perspectiva de un consumidor externo.

## Learning

**Lo que salió como se esperaba:**
- Las 6 interfaces existentes (`Engine`, `Component`, `HostPolicy`, `RuntimeError`, `Val`, `Resolver`) son suficientes y auto-contenidas para soportar escenarios embebidos sin requerir capas abstractas intermedias.
- La implementación de `Debug` para `Component`, `Engine` y `Resolver` resolvió la necesidad ergonómica de formateo e inspección en consumidores externos (`expect_err`, macros de logging, etc.) sin exponer detalles internos de `Payload`.
- Se añadieron métodos de inspección `Component::is_component_model()` y `Component::policy()` que complementan a `Component::is_core_module()`.

**Lo inesperado y ajustes realizados:**
- Se evitó la tentación de crear tipos wrappers sobre `wasmtime::component::Val`; el re-export directo de `Val` resulta ser el camino más eficiente y canónico, compatible con las herramientas nativas del Component Model.
- Se incorporó un doctest compilable en el módulo raíz `crates/runtime/src/lib.rs` que valida en CI la corrección de los snippets de documentación.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --all-targets --workspace`: 0 advertencias.
   - `cargo fmt --check`: 100% limpio.
2. **Documentación:**
   - `cargo doc -p agent-native-runtime --no-deps`: Compila con 0 errores/warnings.
   - Doctest en `crates/runtime/src/lib.rs`: 1 passed.
3. **Pruebas de la API Pública:**
   - `crates/runtime/tests/public_api_audit.rs`: 4 passed; 0 failed.
4. **Workspace Completo:**
   - 124 pruebas automáticas superadas: 124 passed; 0 failed.
