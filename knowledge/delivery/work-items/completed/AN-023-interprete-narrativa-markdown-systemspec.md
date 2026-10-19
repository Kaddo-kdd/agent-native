---
id: AN-023
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-023 — Intérprete narrativa Markdown → SystemSpec"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/cli/src/spec/**
  - crates/cli/src/lib.rs
  - crates/cli/tests/spec_interpreter.rs
source_roadmap_initiative: INIT-04-01
initiative: INI-004
domains:
  - Specification & Knowledge Ingestion
  - Validation & Evolution
related_capabilities:
  - Specification Interpretation
expected_value: >-
  Proveer un parser e intérprete agnóstico de especificaciones en lenguaje natural
  estructurado (Markdown) que extraiga metadatos, componentes, entidades, requisitos y
  contratos WIT en un modelo de datos canónico y fuertemente tipado (`SystemSpec`),
  serializable a JSON y con validaciones fail-explicit sin depender de modelos LLM.
dependencies:
  - AN-003
source_signals:
  - "POC crates/orchestrator/src/interpreter.rs"
  - "Roadmap RM-04: INIT-04-01"
  - "ADR-001: Arquitectura inicial del workspace"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-023 — Intérprete narrativa Markdown → SystemSpec

**Actor y resultado:**
Un desarrollador, agente o pipeline de compilación proporciona un documento de especificación en Markdown y obtiene una estructura de datos canónica `SystemSpec` fuertemente tipada y validada, o un error explícito si la narrativa carece de componentes o requisitos estructurados.

**Current behavior:**
- La CLI (`crates/cli`) era únicamente un scaffolding con `main.rs` sin módulos de especificación u orquestación.
- No existía en el nuevo repositorio una representación tipada de `SystemSpec` ni un parser para transformar narrativas Markdown en especificaciones estructuradas.

**Target behavior:**
- Módulo `crates/cli/src/spec/` con:
  - `model.rs`: Tipos canónicos serializables (`SystemSpec`, `SpecMetadata`, `ComponentSpec`, `RequirementSpec`, `EntitySpec`, `OperationSpec`).
  - `error.rs`: Errores tipados de interpretación (`SpecError`).
  - `interpreter.rs`: Trait `RequirementsInterpreter` y su implementación determinista `MarkdownRequirementsInterpreter`.
- Exposición pública en `crates/cli/src/lib.rs`.
- Suite de pruebas completa que reproduce los casos de prueba de la POC y verifica validaciones fail-explicit.

## Acceptance criteria

### AC-01 — Modelo de Datos `SystemSpec` Canónico
- Estructura `SystemSpec` con `spec_version`, `metadata`, `components`, `requirements`, `entities` y `operations`.
- Implementa `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`.
- Capacidad de serializarse a JSON canónico con indentación y deserializarse sin pérdida de información.

### AC-02 — Interpretación de Componentes, Entidades y Requisitos
- Extrae título del documento desde encabezado `# Title`.
- Parsea componentes con ID explícito en encabezados (`## Componente (`id`)` o `### Componente`).
- Extrae entidades con nombres y lista de campos tipados (`- name: type description` o `- \`name\`: description`).
- Extrae requisitos funcionales con identificadores estables (`- **ID-xxx**: statement`).
- Infiere dependencias entre componentes y asociaciones de interfaces exportadas/importadas.

### AC-03 — Extracción de Contratos WIT desde la Narrativa
- Parsea bloques de código ```wit delimitados dentro de la narrativa.
- Extrae interfaces y firmas de operaciones (`func(param: type) -> return_type`).
- Asocia las operaciones parseadas al componente correspondiente y a su interfaz WIT.

### AC-04 — Validación Fail-Explicit
- Si el documento no contiene componentes identificables, retorna un error explícito `SpecError::MissingComponents` (no inventa componentes ni devuelve estructuras vacías silenciosas).
- Si el documento no contiene requisitos reconocibles, retorna `SpecError::MissingRequirements`.
- Los errores son tipados y derivan de `thiserror`.

### AC-05 — Cobertura y Regresión Offline
- Reproduce los tests de la POC:
  - Narrativa de telemetría satelital (`test_interprets_unseen_custom_narrative`).
  - Rechazo de narrativa incompatible (`test_rejects_incompatible_narrative_without_inventing_components`).
- Pruebas adicionales de parsing de múltiples interfaces, records complejos y serialización JSON determinista.
- Todas las pruebas corren offline sin acceso a red ni llamadas a LLMs.

## Out of scope
- Invocación de LLMs o prompts generativos (pertenece a INIT-04-02).
- Evolución incremental de especificaciones v1 → v2 (pertenece a INIT-04-03).
- Compilación a Semantic IR o WebAssembly (pertenece a `crates/compiler`).

## Validation
```bash
cargo test -p agent-native
cargo test --workspace
```

## Definition of Done
- [x] Modelo `SystemSpec` fuertemente tipado y serializable implementado en `crates/cli/src/spec/model.rs`.
- [x] Trait `RequirementsInterpreter` y parser `MarkdownRequirementsInterpreter` en `crates/cli/src/spec/interpreter.rs`.
- [x] Tipo de error estructurado `SpecError` en `crates/cli/src/spec/error.rs`.
- [x] Módulo `spec` expuesto en `crates/cli/src/lib.rs` conforme a ADR-001.
- [x] Pruebas de la POC y de regresión pasando (7/7 tests en `agent-native`).
- [x] Invariantes de aislamiento ADR-001/ADR-002 verificadas (`runtime` y `compiler` intactos).
- [x] Formato (`cargo fmt`) y lints (`cargo clippy`) limpios sin advertencias.

## Trazabilidad
| Fuente | Referencia |
|---|---|
| Roadmap iniciativa | INIT-04-01 |
| Bloque | RM-04 (Specification, Agents & Build Pipeline) |
| Iniciativa Kaddo | INI-004 |
| ADRs | ADR-001, ADR-002 |
| POC de origen | `_reference/poc-agent-native/crates/orchestrator/src/interpreter.rs` |

## Learning

**Lo que salió como se esperaba:**
- La integración dentro de `crates/cli/src/spec` respeta a la perfección ADR-001: evita la fragmentación temprana del workspace en un quinto crate y permite probar el parser mediante pruebas unitarias e integración directa en `agent-native`.
- El parser dinámico de narrativa recuperó fielmente las capacidades validadas en la POC:
  - Extracción de títulos de documentos Markdown.
  - Reconocimiento de múltiples componentes a partir de encabezados o listas en la visión general.
  - Reconocimiento de entidades con normalización de tipos (`i32`, `bool`, registros).
  - Inferencia de dependencias entre componentes a partir de las declaraciones de requisitos cruzados.
  - Resolución automática de interfaces importadas (`host-services` para llamadas a `kv_get`, `emit_audit`, etc.).
  - Extracción de operaciones desde bloques ```wit, vinculando los tipos de retorno a las entidades declaradas en el documento.
- Fail-explicit: documentos sin componentes o sin requisitos fallan de forma determinista con errores tipados (`SpecError`), impidiendo que el agente genere o invente especificaciones vacías.

**Lo inesperado:**
- La serialización JSON (`SystemSpec::to_json_pretty` y `SystemSpec::from_json_str`) demostró un roundtrip idéntico y sin pérdida de datos, lo cual habilitará la persistencia determinista de especificaciones en el store local de artefactos en INIT-04-05.
- La incorporación del crate `agent-native` como biblioteca (`[lib]` en `Cargo.toml`) permite que los módulos de orquestación futuros (`build::`, `publish::`) cuenten con pruebas de integración independientes sin contaminar el punto de entrada binario `main.rs`.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings en los 4 crates.
   - `cargo fmt -- --check`: Formato estricto cumplido.
2. **Pruebas de la suite `agent-native`:**
   - 3 tests unitarios en `crates/cli/src/spec/interpreter.rs`.
   - 4 tests de integración en `crates/cli/tests/spec_interpreter.rs`.
   - Total: 7 passed; 0 failed.
3. **Workspace Completo:**
   - 82 pruebas pasando en total (7 `agent-native` + 13 `compiler` + 6 `ir` + 56 `runtime`).
