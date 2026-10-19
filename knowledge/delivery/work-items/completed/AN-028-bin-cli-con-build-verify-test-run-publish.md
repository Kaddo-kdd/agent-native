---
id: AN-028
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-028 — Binario CLI unificado con build, verify, test, run y publish"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/cli/Cargo.toml
  - crates/cli/src/main.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/cli.rs
  - crates/cli/src/commands/mod.rs
  - crates/cli/src/commands/build.rs
  - crates/cli/src/commands/verify.rs
  - crates/cli/src/commands/test.rs
  - crates/cli/src/commands/run.rs
  - crates/cli/src/commands/publish.rs
  - crates/cli/src/output.rs
  - crates/cli/src/spec/model.rs
  - crates/cli/tests/cli_e2e.rs
  - knowledge/tech/cli-guide.md
source_roadmap_initiative: INIT-05-01
initiative: INI-005
domains:
  - Distribution & DX
related_capabilities:
  - CLI as Primary Interface
expected_value: >-
  Proveer la CLI binaria unificada `agent-native` con subcomandos canónicos `build`, `verify`, `test`,
  `run` y `publish`, desacoplada de la lógica de dominio mediante `clap`, con soporte de salida
  humana y estructurada JSON (`--format=json`), códigos de salida deterministas y pruebas E2E
  desde subprocesos independientes.
dependencies:
  - AN-026
  - AN-027
source_signals:
  - "Roadmap RM-05: INIT-05-01"
  - "ADR-001: CLI sin lógica de dominio propia"
  - "Decisión Humana D4 (2026-10-19): Contratos CLI, salidas structured y suites E2E"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T12:20:00.000Z
ready_at: '2026-10-19'
---

# AN-028 — Binario CLI unificado con build, verify, test, run y publish

**Actor y resultado:**
Un desarrollador o pipeline de CI ejecuta la herramienta de línea de comandos `agent-native` invocando los subcomandos `build`, `verify`, `test`, `run` y `publish`. La CLI proporciona salida legible humana por defecto o estructurada JSON mediante `--format=json`, reporta códigos de salida deterministas (`0` éxito, `1` fallo de dominio/compilación/test, `2` error de argumentos o I/O), y delega toda lógica de dominio a los crates del workspace (`agent-native-ir`, `agent-native-compiler`, `agent-native-runtime` y módulos de orquestación `spec`/`publish`).

**Acceptance Criteria:**
- **AC-01 (Parser CLI & Subcomandos Canónicos):** La CLI utiliza `clap` (derive API) y expone los 5 subcomandos canónicos:
  - `build`: compila una especificación narrativa (`.md`) o documento Semantic IR (`.json`) generando componente WebAssembly (.wasm) y hash SHA-256. Soporta `--mode` (`synthesizer`, `fixtures`).
  - `verify`: valida la integridad y corrección estática de un archivo Semantic IR (`.json`) o `SystemSpec` (`.md`), emitiendo diagnósticos tipados.
  - `test`: ejecuta suites de aceptación YAML o escenarios declarativos contra un componente Wasm o especificación, retornando resultados de pruebas.
  - `run`: ejecuta una función exportada de un componente Wasm mediante el runtime independiente, pasando argumentos tipados y emitiendo los resultados.
  - `publish`: empaqueta, genera hash SHA-256 y publica atómicamente componentes en el store de versiones local con manifiesto y trazabilidad.
- **AC-02 (Salidas Estructuradas `--format`):** Flag global o por comando `--format <human|json>` (default: `human`). En modo JSON, la salida a `stdout` es estrictamente parseable por herramientas de CI/CD sin texto intercalado.
- **AC-03 (Códigos de Salida Deterministas):**
  - `0`: Éxito total de la operación.
  - `1`: Fallo de validación, error de compilación de componente, aserción de test fallida o error en ejecución de runtime.
  - `2`: Error de sintaxis en argumentos CLI, archivo no encontrado o error de lectura de I/O.
- **AC-04 (Desacoplamiento Arquitectónico ADR-001):** La CLI no contiene lógica de compilación, verificación ni ejecución de bytecode en sus parsers; actúa como adaptador de interfaz cableando las APIs públicas del workspace.
- **AC-05 (Pruebas E2E de Proceso Independiente):** Suite de pruebas de integración (`crates/cli/tests/cli_e2e.rs`) invocando el binario real compilado (`env!("CARGO_BIN_EXE_agent-native")`), verificando casos exitosos, casos negativos, códigos de salida y parsing de salida JSON sin dependencias de red ni LLM real.

**Out of scope:**
- Nuevos crates o empaquetado multi-target (diferido a AN-032).
- Publicación en crates.io o distribución de binarios públicos.
- Invocación de proveedores LLM en red en las pruebas automatizadas de la CLI.

## Learning

**Lo que salió como se esperaba:**
- `clap` (derive) permitió definir una interfaz de línea de comandos tipada y declarativa sin acoplar la lógica de dominio al parser de argumentos.
- Los subcomandos `build`, `verify`, `test`, `run` y `publish` operan tanto con especificaciones narrativas Markdown como con documentos Semantic IR v0.5 de forma transparente.
- Salida JSON estructurada pura: los comandos emiten objetos JSON estrictos en `stdout` en modo `--format json`, garantizando que pipelines de CI puedan parsear resultados y diagnósticos directamente.
- Flujo unificado verificado de extremo a extremo: un único archivo Markdown recorre `verify` -> `build` -> `run` -> `test` -> `publish`, produciendo artefactos reproducibles y byte-idénticos entre build local y almacén de versiones.

**Lo inesperado y ajustes realizados:**
- Invocación de interfaces en el Component Model de Wasmtime: las funciones exportadas bajo interfaces WIT se encuentran indexadas con calificadores (`ops#<func>` o `<paquete>/<interfaz>@<version>#<func>`). En `run.rs` se implementó resolución heurística de calificadores candidatos para permitir que el usuario invoque funciones usando nombres simples (kebab-case o snake_case) sin requerir el namespace completo en CLI.
- Sintetizador de WIT automático: se incorporó `generate_wit_for_component` para permitir que el compilador resuelva automáticamente las interfaces WIT generadas durante compilación al vuelo si no existen en disco.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --all-targets --workspace`: 0 advertencias.
   - `cargo fmt --check`: 100% limpio.
2. **Pruebas de Integración E2E de CLI:**
   - 19 pruebas de integración ejecutadas como subprocesos independientes del OS en `crates/cli/tests/cli_e2e.rs`: 19 passed; 0 failed.
3. **Workspace Completo:**
   - 117 pruebas automatizadas superadas a nivel workspace: 117 passed; 0 failed.
