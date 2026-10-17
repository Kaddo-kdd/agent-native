---
type: extraction-matrix
project_state: ai-assisted
generated_by: implementation-agent
template_version: 1
work_item: AN-002
poc_commit: 26b6a18e2f4a50086d5ee604051154d9ea77d768
---

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

# Matriz de extracción POC → Agent-Native

Mapa decisorio pieza-a-pieza de la POC `_reference/poc-agent-native/` (congelada en commit `26b6a18e`). Ordena RM-02/03/04 del [roadmap](../delivery/roadmap.md) y es prerrequisito de los ADRs bloqueantes (INIT-01-03) y del workspace inicial (INIT-01-01). Vinculada al Work Item [AN-002](../delivery/work-items/in-progress/AN-002-extraction-matrix.md).

**Convenciones:**

- **Decisión:** `migrate` (migrar como está, con pruebas), `redesign` (recuperar comportamiento, reescribir API), `drop` (no se migra; permanece solo en `_reference/`).
- **Destino conceptual:** crate, módulo o carpeta del producto nuevo. "—" para `drop`.
- **Pruebas asociadas:** las pruebas de la POC que validan la pieza. "—" si no las hay en POC.
- **Responsable candidato:** rol/equipo. Orientativo (K3), no vinculante.
- **Riesgo migración:** solo cuando el riesgo no es trivial.

Toda consulta a `_reference/` se hace bajo la política [reference-policy.md](reference-policy.md): lectura selectiva, implementación exclusivamente en el producto nuevo, cita del archivo consultado en el WI derivado.

## 1. Crates principales (3)

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| C-01 | `crates/irtool/src/lib.rs` (2749 LOC) — modelo IR, verifier, compilación WASM, hash SHA-256, impresión WAT | **redesign** | `crates/ir` (modelo + verifier) + `crates/compiler` (backend WASM, hash) | 8 unit tests en `irtool` | IR + compiler | R-03 (acoplamiento) | Separar `ir::` (modelo, verifier) de `compiler::` (backend, hash). `print_wasm_to_wat` queda en `compiler` como utilidad opcional. |
| C-02 | `crates/irtool/src/main.rs` (266 LOC) — CLI `irtool verify/compile` | **drop** | — | — | CLI | — | La CLI unificada `agent-native` sustituye a `irtool`; el binario vive en `crates/cli` (INIT-05-01). |
| C-03 | `crates/orchestrator/src/lib.rs` (1963 LOC) — `SystemSpec`, build, evolución, publicación atómica, SHA-256, puntero activo | **redesign** | capa de orquestación (dentro del bin `cli` o crate `orchestrator` cuando ADR-001 lo justifique) + módulo `publishing` (INIT-04-05) | 9 unit tests en `orchestrator` | Orquestación + QA | R-11 (allocator), R-03 (acoplamiento) | La publicación atómica se extrae a su propio módulo con fronteras claras (ADR-001). `BuildArtifact`/`BuildSummary`/`BuildFailPoint` se preservan como contratos. |
| C-04 | `crates/orchestrator/src/interpreter.rs` (594 LOC) — `MarkdownRequirementsInterpreter`, parser narrativa | **migrate** | módulo `spec::interpret` en orquestación (INIT-04-01) | fixtures `spec/narrative-*.md` + tests indirectos vía orchestrator suite | Orquestación | — | API observable (`SystemSpec` serializable) ya validada por la POC; migrar con pruebas de serialización 1:1. |
| C-05 | `crates/orchestrator/src/agent.rs` (770 LOC) — `ImplementationAgent` trait, `LlmImplementationAgent`, `SynthesizerAgent`, `MockDiagnosticAgent`, `LlmConfig`, `AgentExecutionMetrics` | **redesign** | `LlmProvider` trait + 3 modos (real / fixtures / synthesizer) en orquestación (INIT-04-02) | ninguna aislada en POC; probado end-to-end | Orquestación + LLM adapter | R-12 (confundir offline con generación real) | Documentar explícitamente garantías por modo. El `SynthesizerAgent` queda **nunca** como sustituto de la capacidad generativa. |
| C-06 | `crates/orchestrator/src/main.rs` (297 LOC) — CLI `orchestrator build/evolve` | **drop** | — | — | CLI | — | Comandos se unifican en `agent-native build/publish` (INIT-05-01). |
| C-07 | `crates/runner/src/main.rs` (2278 LOC) — Wasmtime engine, carga/composición WASM, host polyfill con allow-list, executor YAML, reportes | **redesign** | `crates/runtime` (engine + API) + `runtime::testing` (executor YAML) + CLI `agent-native run/test` | 10 unit tests en `runner` + 8 suites YAML en `tests/` | Runtime + QA | R-02 (recompilación por invocación), R-07 (fail-explicit) | Dividir en 3 piezas: (a) carga/invocación tipada, (b) policy loader deny-by-default (INIT-03-05/06-01), (c) executor YAML. La policy "allow-list" ya está en POC — se preserva como contrato invariante. |

## 2. Crates baseline (4)

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| C-08 | `crates/baseline-rust{,-002,-003,-004}/src/lib.rs` (~29–129 LOC c/u) | **drop** | — | — | QA | — | Son implementaciones Rust nativas usadas como punto de comparación de rendimiento en los `BENCHMARK_REPORT_*`. No forman parte del producto. Se re-crearán si se necesitan en RM-08, con su propia iniciativa. |

## 3. Contratos WIT (9)

Todos los `.wit` de `_reference/poc-agent-native/wit/` son **fixtures de dominio de la POC**, no contratos públicos del producto.

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| W-01 | `wit/hydro-sensor.wit` | **migrate** | `tests/fixtures/wit/` | smoke tests varios | QA | — | Fixture para dominio de control de esclusas v1. |
| W-02 | `wit/lock-controller.wit` | **migrate** | `tests/fixtures/wit/` | hidden-002.yaml, smoke-v2.yaml | QA | — | Mismo dominio esclusas, v1. |
| W-03 | `wit/order-processor.wit` | **migrate** | `tests/fixtures/wit/` | public-003.yaml, hidden-003.yaml | QA | — | Dominio orders v3. |
| W-04 | `wit/order.wit` | **migrate** | `tests/fixtures/wit/` | public-003.yaml | QA | — | Interfaz orders. |
| W-05 | `wit/quota-engine.wit` | **migrate** | `tests/fixtures/wit/` | resource-reservation*.yaml | QA | — | Dominio reservations. |
| W-06 | `wit/reservation-coordinator.wit` | **migrate** | `tests/fixtures/wit/` | resource-reservation*.yaml | QA | — | Dominio reservations. |
| W-07 | `wit/shipping-calculator.wit` | **migrate** | `tests/fixtures/wit/` | public-004.yaml, hidden-004.yaml, narrative-005*.yaml | QA | — | Dominio shipping, usado como caso E2E unknown-domain. |
| W-08 | `wit/subscription-billing.wit` | **migrate** | `tests/fixtures/wit/` | smoke-v4.yaml | QA | — | Dominio billing. |
| W-09 | `wit/usage-tracker.wit` | **migrate** | `tests/fixtures/wit/` | smoke-v4.yaml | QA | — | Dominio billing. |

**Importante (riesgo R-08):** ningún `.wit` migrado entra en `wit/public/` del producto. Todos van a `tests/fixtures/wit/` o `examples/`. INIT-02-03 formaliza esta regla con lint CI.

## 4. Suites YAML de aceptación (18 archivos + 1 directorio)

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| Y-01 | `tests/smoke.yaml`, `smoke-v2.yaml`, `smoke-v3.yaml`, `smoke-v4.yaml` (4) | **migrate** | `tests/acceptance/smoke/` | self | QA | — | Smoke tests por versión de IR. |
| Y-02 | `tests/public.yaml`, `public-002.yaml`, `public-003.yaml`, `public-004.yaml` (4) | **migrate** | `tests/acceptance/public/` | self | QA | — | Suite pública por iteración. |
| Y-03 | `tests/hidden-002.yaml`, `hidden-003.yaml`, `hidden-004.yaml`, `hidden-narrative-005.yaml` (4) | **migrate** | `tests/acceptance/hidden/` | self | QA | — | Suite "hidden" — casos no mostrados al agente generador; validan generalización. |
| Y-04 | `tests/narrative-005.yaml`, `narrative-005-e3.yaml`, `narrative-005-e4.yaml` (3) | **migrate** | `tests/acceptance/narrative/` | self | QA | — | Validan el flujo narrativa → componente del escenario unknown-domain. |
| Y-05 | `tests/resource-reservation.yaml`, `resource-reservation-v2.yaml`, `resource-reservation-v3.yaml` (3) | **migrate** | `tests/acceptance/reservations/` | self | QA | — | Validan evolución v1→v2→v3 del dominio reservations. |
| Y-06 | `tests/suites-poc002/` (dir) | **drop** | — | — | QA | — | Agregado histórico de POC-002. La equivalencia funcional del MVP ya cubre su alcance con las suites anteriores. |

**Total suites a migrar:** 18 archivos YAML (acepta `drop` del directorio `suites-poc002/`).

## 5. Specs IR (schemas + programas)

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| S-01 | `ir/ir.schema.json`, `ir.schema.v2.json`, `ir.schema.v3.json`, `ir.schema.v4.json` (4) | **migrate** | `crates/ir/schemas/` | consumen directamente los tests `irtool` | IR | R-09 (divergencia v0.5↔v0.6) | Son la gramática validada por `verify_ir`. El schema v0.5 queda como baseline estable; versiones previas se preservan como histórico documentado. |
| S-02 | `ir/program*.ir.json` (5 archivos) + `smoke-v*.ir.json` (3) + `order-005.ir.json`, `shipping-005.ir.json` | **migrate** | `tests/fixtures/ir/` | consumen `irtool`/`runner` suites | QA | — | Programas IR de ejemplo para pruebas de regresión. |

## 6. Specs narrativos y Work Items

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| N-01 | `spec/semantic-ir-v0.5-spec.md`, `semantic-ir-v0.5.schema.json`, `system-spec.json`, `system-spec.schema.json` | **migrate** | `knowledge/tech/semantic-ir-v0.5/` (docs) + `crates/ir/schemas/` (schemas) | — | IR | — | Documentación canónica de Semantic IR v0.5 que debe sobrevivir. El `.md` queda como fuente para redactar la variante nueva. |
| N-02 | `spec/narrative-005.md`, `narrative-resource-reservation.md`, `narrative-subscription-service.md` (3) | **migrate** | `examples/narratives/` | consumidas por suites `narrative-*` | Orquestación | — | Narrativas de dominio ejecutables por el intérprete (C-04). |
| N-03 | `spec/task*.yaml` (4) | **redesign** | `examples/work-items/` | — | Orquestación | — | Pre-formato de Work Items de la POC; se reinterpretan al formato Kaddo del nuevo producto. |
| N-04 | `spec/work-items-{e2,e3,e4,res}/` (4 dirs) | **redesign** | `examples/work-items/` | consumidas por `parse_work_items_consolidated` | Orquestación | — | Work Items de dominio para las iteraciones; se adaptan al formato Kaddo. |

## 7. Runs E2E y evidencia

Los 35 directorios `runs/` son **registros de ejecución**, no código.

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| R-01 | `runs/unknown-domain-e2e/` | **migrate** | `tests/e2e/unknown-domain/` (como fixture reproducible) + `docs/evidence/poc/` (evidencia histórica) | referenciado por suites `narrative-*` | QA + Runtime | — | **Prueba central** del flujo LLM real → IR → WASM → aceptación en dominio no preprogramado. Debe reproducirse en el MVP. |
| R-02 | `runs/{e1,e2,e3,e4}-*/` + `poc002-*`, `poc003-*`, `poc004-*`, `poc005-*/` (28 dirs) | **drop** | — | — | — | — | Registros de ejecuciones por iteración de la POC. Se dejan en `_reference/` como evidencia histórica consultable; no se migran. |
| R-03 | `runs/rust-baseline*/` (4 dirs) | **drop** | — | — | — | — | Resultados de `baseline-rust*`. Se re-ejecutarán en RM-08 (INIT-08-02) con el harness propio del producto. |
| R-04 | `runs/standalone-validation/` | **migrate** | `docs/evidence/standalone/` | — | QA | — | Prueba de ejecución sin cargo/LLM, antecedente directo del objetivo RM-05. Se preserva como fuente de verificación. |
| R-05 | `runs/toolchain-freeze*.sha256` (5) | **drop** | — | — | — | — | Hashes de toolchain por iteración POC; no reutilizables (nueva toolchain fijada en INIT-01-01/ADR-003). |

## 8. Reportes y Release Candidate

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| B-01 | `BENCHMARK_REPORT_005.md` | **migrate** | `docs/evidence/poc/BENCHMARK_REPORT_005.md` | — | QA | R-10 (afirmaciones sin medición) | Fuente de contexto para RM-08. Sus cifras NO se adoptan como SLAs; INIT-08-02 produce la línea base propia. |
| B-02 | `BENCHMARK_REPORT{,_002,_003,_004}.md` (4) | **drop** | — | — | — | — | Reemplazados por `_005`. Permanecen en `_reference/` para consulta histórica. |
| B-03 | `RELEASE_CANDIDATE_v0.1.0.md` | **migrate** | `docs/evidence/poc/RELEASE_CANDIDATE_v0.1.0.md` | — | QA | — | Documenta garantías y limitaciones del cierre POC. Fuente para los criterios de salida del MVP del producto nuevo. |

## 9. Infraestructura y otros

| ID | Pieza POC | Decisión | Destino | Pruebas asociadas | Responsable | Riesgo | Notas |
|---|---|---|---|---|---|---|---|
| X-01 | `Cargo.toml`, `Cargo.lock` (workspace POC) | **drop** | — | — | — | — | El workspace nuevo se crea en INIT-01-01 con 3 crates y bin `cli`; no se hereda estructura. |
| X-02 | `LICENSE` | **drop** | — | — | — | — | El producto nuevo ya tiene su propio `LICENSE` (MIT) en la raíz. |
| X-03 | `build/` (vacío en POC) | **drop** | — | — | — | — | Directorio de salida de build de la POC. No aplica. |
| X-04 | `knowledge/` (interno POC) | **drop** | — | — | — | — | Conocimiento Kaddo propio de la POC; el producto nuevo ya tiene su propio `knowledge/`. |

## Totales por decisión

| Decisión | Count | Comentario |
|---|---|---|
| **migrate** | 24 | 1 pieza (C-04 intérprete) + 9 WIT + 18 suites YAML + 2 schemas y 5 programas IR agregados por fila (S-01, S-02) + 3 narrativas (N-02) + 1 doc Semantic IR (N-01) + 2 runs (R-01, R-04) + 2 reportes (B-01, B-03). Las filas S-01/S-02 agrupan múltiples archivos del mismo tipo. |
| **redesign** | 6 | C-01 (irtool→ir+compiler), C-03 (orchestrator lib→orq+publishing), C-05 (agent→LlmProvider), C-07 (runner→runtime), N-03 (tasks), N-04 (work-items) |
| **drop** | 11 | C-02, C-06 (CLIs POC), C-08 (4 baseline-rust agregados), Y-06, R-02, R-03, R-05, B-02, X-01, X-02, X-03, X-04 |

**Concentración relevante:**

- **IR + verifier + compiler** se agrupan en 2 crates (`ir`, `compiler`) a partir de un único `irtool/lib.rs` de 2749 LOC — mayor riesgo de rework de API; ADR-001 debe fijar los límites antes de abrir INIT-02-01/02-02.
- **Orchestrator** se descompone en 3 responsabilidades (intérprete, LLM adapter, publishing) de un único `orchestrator/lib.rs` de 1963 LOC — mismo riesgo; ADR-001 lo cubre.
- **Runner** se descompone en 3 piezas (engine, policy loader, executor YAML) de un único `runner/main.rs` de 2278 LOC — tratado en INIT-03-01/03-05.
- Toda la **evidencia histórica** (benchmarks, release candidate, runs) permanece en `_reference/` como "consultable, no migrada" salvo las 4 piezas específicas listadas en `migrate`.

## Dependencias con el roadmap

- **INIT-02-01** (`ir`): consume C-01 (parte verifier/model) + S-01 (schemas) + S-02 (programas) + N-01 (docs IR).
- **INIT-02-02** (`compiler`): consume C-01 (parte backend WASM + hash).
- **INIT-02-03** (`wit/`): consume W-01..W-09 como fixtures en `tests/fixtures/wit/`.
- **INIT-02-04** (determinismo + hash): consume la lógica SHA-256 de C-01.
- **INIT-03-01** (`runtime` engine): consume C-07 (parte carga/invocación).
- **INIT-03-03** (executor YAML): consume C-07 (parte executor) + Y-01..Y-05.
- **INIT-03-04** (errores host): consume C-07 (parte propagación tipada; parcialmente ausente en POC — hay que rediseñar).
- **INIT-03-05** (deny-by-default): formaliza la allow-list ya presente en C-07.
- **INIT-04-01** (narrativa): consume C-04 + N-02.
- **INIT-04-02** (LLM adapter): consume C-05 con separación explícita de modos.
- **INIT-04-03** (evolución): consume C-03 (parte evolución) + N-03/N-04 + Y-04/Y-05.
- **INIT-04-05** (publicación): consume C-03 (parte publishing atómico), ya aislada por ADR-001.

## Referencias

- Work Item: [AN-002](../delivery/work-items/in-progress/AN-002-extraction-matrix.md)
- Roadmap: [knowledge/delivery/roadmap.md](../delivery/roadmap.md) — bloque RM-01, iniciativa INIT-01-02
- Política de referencia: [reference-policy.md](reference-policy.md) — AN-001
- Mapa del codebase: [codebase.md](codebase.md)
