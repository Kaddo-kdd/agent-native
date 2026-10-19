---
id: AN-026
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-026 — Publicación atómica, versionamiento e integridad"
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
code:
  - crates/cli/src/publish/**
  - crates/cli/src/lib.rs
  - crates/cli/tests/atomic_publish.rs
source_roadmap_initiative: INIT-04-05
initiative: INI-004
domains:
  - Validation & Evolution
  - Distribution & DX
related_capabilities:
  - Versioned Publication & Evolution
expected_value: >-
  Proveer un store de artefactos filesystem local (`versions/build-{timestamp}`),
  generación de manifiestos versionados con hashes SHA-256 (ADR-004), puntero activo
  (`active.json`) con promoción atómica y rollback ante fallos parciales o staging interrumpido,
  desacoplado limpiamente del crate compiler (ADR-001).
dependencies:
  - AN-023
  - AN-024
  - AN-025
source_signals:
  - "POC crates/orchestrator/src/lib.rs (atomic_replace_file, recover_and_verify_dist_state, build_all_components_with_options)"
  - "Roadmap RM-04: INIT-04-05"
  - "ADR-001: Arquitectura inicial del workspace (publicación en orchestrator/CLI, no en compiler)"
  - "ADR-004: Trazabilidad determinista de componentes y SHA-256"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-026 — Publicación atómica, versionamiento e integridad

**Actor y resultado:**
Un desarrollador o pipeline de CI/CD compila un `SystemSpec` a través de un `LlmProvider` y publica los artefactos WASM e IR generados en un store de versiones (`versions/build-{timestamp}`). La publicación genera un `manifest.json` inmutable con hashes SHA-256, promueve atómicamente el puntero `active.json` y permite la recuperación o verificación de integridad (`recover_and_verify_dist_state`), asegurando que cualquier fallo intermedio durante el staging o la copia no corrompa el estado activo ni exponga versiones parciales.

**Current behavior:**
- AN-023 proveyó el modelo `SystemSpec` y el intérprete de narrativa Markdown.
- AN-024 proveyó el contrato `LlmProvider` y los modos de generación (`real`, `fixtures`, `synthesizer`).
- AN-025 proveyó el cálculo de `SpecDiff` y el reconciliador acumulativo `reconcile_spec`.
- No existía un gestor de publicación atómica ni store de versiones en disco para empaquetar y versionar los componentes compilados de forma segura con puntero `active.json`.

**Target behavior:**
- Módulo `crates/cli/src/publish/` con:
  - `atomic.rs`: Operación de reemplazo atómico de archivos (`atomic_replace_file`) con preservación de estado e idempotencia.
  - `store.rs`: Estructuras `BuildArtifact`, `BuildSummary`, `VersionManifest`, `ActivePointer`.
  - `publisher.rs`: Pipeline de compilación y publicación atómica (`build_and_publish_components`) orquestando `LlmProvider`, `agent-native-compiler`, staging en `.staging/`, escritura de `versions/build-{id}/manifest.json`, promoción atómica de `active.json` y verificación de integridad (`recover_and_verify_dist_state` y `resolve_active_component_artifact`).
  - `error.rs`: Errores tipados `PublishError`.
- Exposición en `crates/cli/src/lib.rs` como módulo `publish`.
- Pruebas de integración offline en `crates/cli/tests/atomic_publish.rs`:
  - Publicación atómica completa de $v_1$.
  - Transición y promoción atómica a $v_2$ preservando $v_1$ en el directorio `versions/`.
  - Simulación de fallo en staging o copia: el puntero activo previo no se corrompe y `.staging/` se recupera limpiamente.
  - Verificación de integridad SHA-256 de los artefactos publicados y detección de alteraciones.

## Acceptance criteria

### AC-01 — Store Local de Versiones
- Directorio estructurado bajo `out_dir`:
  - `versions/build-{timestamp}/`: Directorio inmutable que contiene `<component>.wasm`, `<component>.ir.json` y `manifest.json`.
  - `active.json`: Puntero canónico que contiene `active_version`, `promoted_at`, `manifest_path` y la lista de artefactos con sus hashes SHA-256.

### AC-02 — Promoción Atómica y Rollback
- La publicación utiliza un staging temporal (`.staging/` y `.active.json.tmp`).
- La promoción de `active.json` se realiza mediante `atomic_replace_file`.
- Ante cualquier error durante la síntesis, compilación o copia de versiones, la versión activa previa se mantiene inalterada y los temporales se limpian sin dejar estados huérfanos.

### AC-03 — Integridad SHA-256 y Recuperación
- `recover_and_verify_dist_state(out_dir)` verifica los hashes SHA-256 de todos los artefactos declarados en el manifiesto activo contra los bytes reales en disco.
- Si un archivo binario fue alterado o falta en disco, falla explícitamente reportando la discrepancia de hash.
- `resolve_active_component_artifact(out_dir, component_id)` resuelve la ruta canónica del binario `.wasm` a partir del puntero activo.

### AC-04 — Desacoplamiento Limpio (ADR-001)
- `agent-native-compiler` permanece completamente agnóstico al store y al filesystem de publicación.
- Toda la orquestación de publicación reside en `crates/cli/src/publish/`.

### AC-05 — Cobertura y Regresión Offline
- Pruebas automatizadas en `crates/cli/tests/atomic_publish.rs` que validan el ciclo de vida completo, detección de corrupción de hash y preservación de estado ante interrupciones.

## Out of scope
- Registries OCI remotos o almacenamiento cloud S3/Blob (diferidos a RM-05).
- Firma criptográfica de manifiestos con claves privadas (corresponde a RM-06).

## Validation
```bash
cargo test -p agent-native
cargo test --workspace
cargo clippy --all-targets --workspace
cargo fmt -- --check
```

## Traceability

| Elemento | Ubicación |
|---|---|
| Reemplazo atómico de archivos | `crates/cli/src/publish/atomic.rs` |
| Modelos de manifiestos y puntero activo | `crates/cli/src/publish/store.rs` |
| Pipeline de build y publicación | `crates/cli/src/publish/publisher.rs` |
| Errores de publicación | `crates/cli/src/publish/error.rs` |
| Exportación pública en biblioteca | `crates/cli/src/publish/mod.rs`, `crates/cli/src/lib.rs` |
| Pruebas de integración offline | `crates/cli/tests/atomic_publish.rs` |

## Learning

**Lo que salió como se esperaba:**
- Separación arquitectónica limpia (ADR-001): la publicación vive en `agent_native::publish` y no contamina el crate `agent-native-compiler`.
- Inmutabilidad de versiones: cada publicación exitosa genera un directorio aislado `versions/build-{nanos}` con sus artefactos y manifiesto correspondiente, permitiendo rollback inmediato re-apuntando `active.json`.
- Integridad verificada criptográficamente (ADR-004): `recover_and_verify_dist_state` recalcula y valida los hashes SHA-256 de todos los binarios `.wasm` contra el manifiesto activo, detectando cualquier corrupción o alteración intencional.
- Resiliencia y atomicidad: las pruebas de fail-points (`DuringVersionCopy` y `DuringActivePromote`) demostraron que ante fallos en staging o en copia, el puntero activo original permanece 100% intacto y el directorio `.staging/` es recolectado.

**Lo inesperado:**
- La resolución de interfaces WIT durante la compilación multi-componente aprovecha el `CompileContext` desacoplado introducido en AN-016, permitiendo a los pipelines especificar raíces de búsqueda personalizadas para contratos externos sin depender del cwd de ejecución.
- El total de pruebas del workspace alcanzó 96 pruebas en el workspace principal y 118 pruebas globales combinadas con los ejemplos de AWS Lambda.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --all-targets --workspace`: 0 warnings.
   - `cargo fmt -- --check`: 100% conforme.
2. **Pruebas de la suite `agent-native`:**
   - 3 pruebas unitarias spec.
   - 4 pruebas de integración spec interpreter.
   - 5 pruebas de integración agent providers.
   - 6 pruebas de integración spec evolution.
   - 3 pruebas de integración atomic publish.
   - Total `agent-native`: 21 passed; 0 failed.
3. **Workspace y Ejemplos:**
   - 96 pruebas del workspace principal superadas.
   - 22 pruebas de AWS Lambda superadas.
   - Total global: 118 passed; 0 failed.
