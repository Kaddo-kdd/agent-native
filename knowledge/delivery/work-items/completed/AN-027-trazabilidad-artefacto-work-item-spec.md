---
id: AN-027
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-027 — Trazabilidad artefacto ↔ Work Item / Spec"
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
code:
  - crates/cli/src/publish/store.rs
  - crates/cli/src/publish/publisher.rs
  - crates/cli/src/publish/mod.rs
  - crates/cli/tests/publish_traceability.rs
source_roadmap_initiative: INIT-04-04
initiative: INI-004
domains:
  - Validation & Evolution
related_capabilities:
  - Versioned Publication & Evolution
expected_value: >-
  Incorporar metadatos de trazabilidad en los manifiestos de publicación (`source_wi`,
  `source_spec_hash`, `source_git_commit`) para reconstruir el origen de cada artefacto
  publicado de forma autónoma y auditable, sin acoplamiento obligatorio a Kaddo.
dependencies:
  - AN-026
source_signals:
  - "Roadmap RM-04: INIT-04-04"
  - "Business Goal: Decisiones basadas en evidencia y trazabilidad determinista"
  - "ADR-004: Trazabilidad determinista de componentes y SHA-256"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-027 — Trazabilidad artefacto ↔ Work Item / Spec

**Actor y resultado:**
Un desarrollador, auditor o sistema de despliegue inspecciona `active.json` o cualquier `manifest.json` en el store de versiones y obtiene la trazabilidad exacta del origen del artefacto: identificadores de Work Items aplicados (`source_wi`), hash criptográfico SHA-256 de la especificación fuente (`source_spec_hash`), y hash opcional de commit git (`source_git_commit`), verificando la procedencia determinista sin requerir conexión a un servidor externo o a Kaddo.

**Current behavior:**
- Tras AN-026, `VersionManifest` y `ActivePointer` almacenan `spec_version`, `spec_source` y la lista de artefactos (`wasm_file`, `sha256`, `ir_file`, etc.).
- No se calculaba ni registraba el hash SHA-256 canónico del `SystemSpec` ni la lista estructurada de Work Items que dieron origen a la versión.

**Target behavior:**
- Extender `crates/cli/src/publish/store.rs`:
  - Agregar estructura de metadatos de trazabilidad `SpecTraceability`:
    - `source_wi: Vec<String>`
    - `source_spec_hash: String`
    - `source_git_commit: Option<String>`
  - Añadir `traceability: SpecTraceability` a `VersionManifest` y `ActivePointer`.
- Actualizar `crates/cli/src/publish/publisher.rs`:
  - Calcular automáticamente `source_spec_hash` a partir de la serialización JSON canónica del `SystemSpec`.
  - Extraer los `source_wi` a partir de `spec.metadata.source_files` y fuentes de requisitos.
  - Soportar parámetro explícito `git_commit` en `PublishOptions` y detección de entorno `AGENT_NATIVE_GIT_COMMIT`.
- Pruebas automatizadas en `crates/cli/tests/publish_traceability.rs`:
  - Comprobación de que `manifest.json` y `active.json` contienen los metadatos de trazabilidad esperados.
  - Comprobación de que la alteración del `SystemSpec` modifica deterministamente el `source_spec_hash`.
  - Verificación de extracción de `source_wi` sin depender de herramientas externas.

## Acceptance criteria

### AC-01 — Metadatos de Trazabilidad en Manifiestos
- `VersionManifest` y `ActivePointer` serializan el bloque `traceability`:
  - `source_wi`: lista de archivos fuente o identificadores de Work Items aplicados.
  - `source_spec_hash`: digest SHA-256 en formato hex del `SystemSpec`.
  - `source_git_commit`: commit opcional obtenido de parámetro o entorno.

### AC-02 — Cálculo Determinista de Hash de Especificación
- El hash `source_spec_hash` se calcula mediante `sha256_hex` sobre la representación JSON canónica del `SystemSpec`, garantizando reproducibilidad.

### AC-03 — Autonomía sin Dependencia Externa
- La trazabilidad se resuelve 100% de manera estática y offline a partir de los datos del `SystemSpec` y el filesystem local, sin requerir APIs ni Kaddo en runtime.

### AC-04 — Cobertura y Regresión
- Pruebas de integración en `crates/cli/tests/publish_traceability.rs`.
- Verificación de linteo, formato y compilación estricta en el workspace.

## Out of scope
- Sincronización bidireccional con issues de GitHub o Jira.

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
| Estructura de trazabilidad | `crates/cli/src/publish/store.rs` |
| Pipeline de cálculo de origen y hashes | `crates/cli/src/publish/publisher.rs` |
| Exportación pública en biblioteca | `crates/cli/src/publish/mod.rs` |
| Pruebas de integración de trazabilidad | `crates/cli/tests/publish_traceability.rs` |

## Learning

**Lo que salió como se esperaba:**
- Trazabilidad criptográfica autónoma (ADR-004): el origen de cada publicación queda sellado con el digest SHA-256 de la especificación `source_spec_hash`. Cualquier mutación en la narrativa, los tipos o los requisitos resulta en un hash diferente.
- Cobertura completa de procedencia (`source_wi`): combina tanto los archivos de especificación base como las referencias a Work Items individuales declaradas en las fuentes de los requisitos (`req.source.ref_target`).
- Independencia total de servicios externos: no existe acoplamiento a APIs de Kaddo, GitHub o bases de datos; la trazabilidad viaja embebida en los artefactos `manifest.json` y `active.json`.

**Lo inesperado:**
- Al utilizar la estructura `PublishOptions`, pudimos evitar mutar variables de entorno en pruebas concurrentes o requerir bloques `unsafe`, manteniendo la estricta política `unsafe_code = "deny"` de todo el workspace.
- El total de pruebas del workspace alcanzó 98 pruebas en el workspace principal y 120 pruebas globales combinadas con los ejemplos de AWS Lambda.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --all-targets --workspace`: 0 warnings.
   - `cargo fmt -- --check`: 100% limpio.
2. **Pruebas de la suite `agent-native`:**
   - 3 pruebas unitarias spec.
   - 4 pruebas de integración spec interpreter.
   - 5 pruebas de integración agent providers.
   - 6 pruebas de integración spec evolution.
   - 3 pruebas de integración atomic publish.
   - 2 pruebas de integración publish traceability.
   - Total `agent-native`: 23 passed; 0 failed.
3. **Workspace y Ejemplos:**
   - 98 pruebas del workspace principal superadas.
   - 22 pruebas de AWS Lambda superadas.
   - Total global: 120 passed; 0 failed.
