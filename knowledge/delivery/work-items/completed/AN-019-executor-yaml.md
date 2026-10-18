---
id: AN-019
type: feature
status: completed
title: Executor de aceptación YAML (hito M3)
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/runtime/src/testing/mod.rs
  - crates/runtime/src/testing/suite.rs
  - crates/runtime/src/testing/stub.rs
  - crates/runtime/src/testing/host_ctx.rs
  - crates/runtime/src/testing/executor.rs
  - crates/runtime/src/testing/report.rs
  - crates/runtime/src/testing/errors.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/Cargo.toml
  - tests/suites/
  - crates/runtime/tests/executor_yaml.rs
  - knowledge/tech/poc-equivalence-matrix.md
source_roadmap_initiative: INIT-03-03
source_work_item_candidate:
  - WI-CANDIDATE-03-03-01
  - WI-CANDIDATE-03-03-02
  - WI-CANDIDATE-03-03-03
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Validation & Evolution
related_capabilities:
  - Acceptance Testing & Regression
  - Independent Runtime Execution
expected_value: >-
  Executor declarativo YAML + registry de stubs tipados Rust que corre suites
  del POC adaptadas mínimamente contra componentes WASM producidos por
  agent-native-compiler. Cierra INIT-03-03, INI-003 6/6, y habilita la
  declaración del hito **M3 — Runtime autónomo con regresión POC**.
risks:
  - >-
    R-E-01: closures de wasmtime::Linker exigen Send+Sync. El host_state y
    call_log deben vivir en Store<T>::data, no en captures. Confirmado por probe
    de viabilidad.
  - >-
    R-E-02: duplicar lógica de invoke.rs. Mitigación: añadir un extension point
    en Component (`invoke_with_linker` o `invoke_with_host_ctx`) que reutilice
    policy check + resolución WIT; executor consume ese extension point.
  - >-
    R-E-03: suites del POC usan ops que requieren el emisor del POC (p. ej. v0.3
    record returns con host state). Mitigación: migrar incrementalmente,
    documentar fallos como skips tipados en la matriz.
dependencies:
  - AN-009
  - AN-010
  - AN-011
  - AN-013
  - AN-014
  - AN-015
source_signals:
  - 'Roadmap INIT-03-03: 10 pruebas runner POC verdes como exit criterion'
  - 'Matriz C-07: executor YAML del POC runner'
  - 'AN-007 Learning: equivalencia integral esclusas v1→v2 va a M4'
  - >-
    AN-015 Learning: HostPolicy tiene allow_wit_interface/operation; stubs
    registrados aquí NO autorizan automáticamente
  - >-
    Probe viability (2026-10-18): confirmado que Core WASM import se resuelve
    con wasmtime::Linker::func_wrap + Store<HostCtx>; stubs reciben args,
    consultan state, retornan resultado, y la llamada queda registrada en orden
decision_candidates: []
related_decisions:
  - ADR-002
  - ADR-004
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-18T00:00:00.000Z
updated_at: 2026-10-18T00:00:00.000Z
ready_at: '2026-10-18'
completed_at: '2026-10-18'
---

# AN-019 — Executor de aceptación YAML (hito M3)

**Actor y resultado:**
Un contribuidor ejecuta `cargo test -p agent-native-runtime --test executor_yaml` y las suites del POC — adaptadas con cambios mínimos — pasan contra componentes compilados por `agent-native-compiler`, usando stubs declarativos en YAML + implementados en Rust como funciones del Linker. `SuiteReport` reporta resultados por caso sin paniquear; una matriz POC → nuevas pruebas documenta explícitamente la equivalencia (10 pruebas runner POC + muestreo de suites YAML).

## Decisiones arquitectónicas incorporadas tras aprobación

Las 4 OQs del draft original quedan cerradas con las elecciones del usuario:

- **OQ-01 crate YAML:** `serde_yaml2` como primer candidato. Si incompat con Rust 1.85 o la deserialización falla contra fixtures POC, se presenta alternativa (`serde_yml` fork activo) **antes** de integrarla.
- **OQ-02 migración de fixtures:** adaptación mínima. Se conservan `id`, `requirement`, nombres de función, argumentos, `expected`, `host_state` y `expected_host_calls` tal cual la POC. Cualquier cambio semántico se documenta en la matriz de equivalencia.
- **OQ-03 stubs:** declarados en YAML bajo clave nueva `host_stubs:` + implementados en Rust como `wasmtime::Linker::func_wrap`. **La declaración de stub NO otorga autorización**; `HostPolicy` sigue siendo la única fuente de verdad para autorización. Un import autorizado por policy sin stub registrado produce error tipado (`StubMissing`); un stub registrado sin autorización produce `UnauthorizedImport`.
- **OQ-04 fallas:** `run_suite` devuelve `SuiteReport` por caso. Ningún caso de usuario produce `panic!`; la integración de pruebas hace `assert_eq!(report.failed, 0)`.

### Items nuevos de alcance aprobados

- **Extension point en Component**: nuevo método `Component::invoke_with_host_ctx<T>` que reutiliza el pre-check de policy + resolución de función de `invoke.rs`, pero recibe un `wasmtime::Linker<T>` ya poblado por el executor. `invoke` original intacto. **Sin duplicar lógica.**
- **Compatibilidad `Component::invoke`**: no cambia su firma ni semánticas. El código existente de AN-009/14/15 queda intocado.
- **Stubs soportados**: `StateReader { key_arg }`, `StateWriter { key_arg, value_arg }`, `Noop { returns }`. **Firma exacta exigida**: `(i32) -> i32`, `(i32, i32) -> i32`, `() -> i32` respectivamente. Cualquier firma distinta, state faltante o stub ausente produce `StubError` tipado.
- **Deny-by-default preservado**: autorización vía `HostPolicy` + registro de stubs son procesos separados.
- **Aislamiento entre casos**: cada caso construye un `Store<HostCtx>` fresh con `host_state` cargado desde el YAML. El `call_log` también se reinicia. **Cero estado leaked entre cases.**
- **Comparaciones**: primitivos (`S32`, `Bool`), records (por campo nombrado), y `expected_host_calls` por **función + args + orden exacto**.
- **Casos negativos obligatorios** en la suite de tests: YAML inválido, op inexistente, import no autorizado, stub faltante, firma incompatible, error de host (trap simulado), resultado incorrecto. Cada uno produce la variante correspondiente de error del executor.
- **Matriz de equivalencia POC → nuevas pruebas**: nuevo documento `knowledge/tech/poc-equivalence-matrix.md` con una fila por cada una de las 10 pruebas unitarias de `runner/src/main.rs` del POC, indicando: nombre POC, mecanismo nuevo (unit test / executor_yaml case), estado (ported / superseded / out-of-scope). El número de cases **no sustituye** esta matriz como evidencia.
- **Dependencias YAML/Serde** del módulo `testing` se declaran como **dependencias normales** del runtime si el módulo es siempre público; si se opta por feature flag, CI tiene un paso explícito que compila **y ejecuta** la feature. Decisión en implementación: módulo público siempre, deps normales (más simple, CI implícito).

## Current behavior (verificado)

- No existe módulo `testing` en el workspace.
- `Component::invoke` instancia con `Linker::new(engine)` vacío (línea 59 y 125 de `invoke.rs`). No hay forma de inyectar stubs sin tocar esa función.
- Las 18 suites YAML POC (`_reference/poc-agent-native/tests/*.yaml`) no se ejecutan contra el runtime nuevo.
- `HostPolicy` ya soporta `allow_core`/`allow_wit_interface`/`allow_wit_operation` (AN-015); el executor las usa sin cambios.

## Target behavior

### API pública `agent-native-runtime::testing`

```rust
pub mod suite;      // Parser + estructuras de datos
pub mod stub;       // HostStubRegistry, StubKind, StubError
pub mod host_ctx;   // HostCtx<T> para Store<T>
pub mod executor;   // run_suite, run_case
pub mod report;     // SuiteReport, CaseResult, CaseStatus
pub mod errors;     // ExecutorError taxonomía

pub use suite::{Suite, Case, parse_suite};
pub use stub::{HostStubRegistry, StubKind};
pub use executor::{run_suite, RunOptions};
pub use report::{SuiteReport, CaseResult, CaseStatus};
pub use errors::ExecutorError;
```

### Extension point en `Component`

```rust
impl Component {
    /// Invokes a function with a caller-provided wasmtime Linker.
    /// Reuses the same policy pre-check and function resolution as
    /// `invoke`; the only difference is that the Linker may contain
    /// registered host functions.
    pub fn invoke_with_host_ctx<T>(
        &self,
        func_name: &str,
        args: &[Val],
        host_ctx: T,
        build_linker: impl FnOnce(&wasmtime::Engine, &mut wasmtime::Linker<T>) -> Result<(), wasmtime::Error>,
    ) -> Result<(Vec<Val>, T), RuntimeError> {
        /* policy check (shared) + custom linker + instantiate + call */
    }
}
```

### Formato YAML extendido

```yaml
suite: KAN-POC-003-PUBLIC
spec_version: 3
host_stubs:
  - interface: host              # Core WASM module OR WIT interface id
    operation: kv_get
    kind: state_reader
    key_arg: 0                   # arg index to look up in host_state.kv
  - interface: host
    operation: emit_audit
    kind: noop
    returns: 1

cases:
  - id: E1-003-P01
    requirement: ORD-001
    function: calculate_order
    args: [101, -1, 1, 100, 1, 500]
    host_state:
      kv:
        101: 10
    expected:
      status_code: 1
      subtotal_cents: 0
    expected_host_calls: []
```

### Fixture scope mínimo

Al menos 10 cases totales distribuidos en 3-5 suites:

- `tests/suites/smoke.yaml` (3 cases) — primitivos sin host.
- `tests/suites/smoke-v2.yaml` (2+ cases) — records sin host.
- `tests/suites/public-003-minimal.yaml` (3+ cases) — con `host_stubs` + `host_state`.
- `tests/suites/negative/` — casos adversariales específicos (YAML inválido, stub ausente, etc.).

### Matriz de equivalencia

`knowledge/tech/poc-equivalence-matrix.md` con una fila por cada de las 10 pruebas unit de `_reference/poc-agent-native/crates/runner/src/main.rs`:

| POC test | Nuevo mecanismo | Status |
|---|---|---|
| test_1 | `tests/executor_yaml.rs::smoke_v01_passes` | ported |
| test_2 | AN-007 `poc_regression::valid_v0_2_...` | superseded (ya ported en RM-02) |
| ... | ... | ... |

Status posibles: `ported` (reproducido), `superseded` (ya reproducido en WI previo), `out-of-scope-M3` (requiere INIT-06-01 host provider; se reserva para RM-04/06).

## Acceptance criteria

1. **Módulo `agent-native-runtime::testing`** público, con 7 submódulos (`suite`, `stub`, `host_ctx`, `executor`, `report`, `errors`, `mod`).
2. **`Component::invoke_with_host_ctx`** nuevo, reutiliza policy + resolución WIT de `invoke.rs`, no la duplica.
3. **`Component::invoke` sin cambios** de firma ni semántica. Los 43 tests previos siguen verdes.
4. **≥10 cases YAML ejecutándose verdes** distribuidos en ≥3 suites, incluyendo al menos 1 suite con `host_stubs`.
5. **Stubs implementados**: `StateReader`, `StateWriter`, `Noop`, cada uno con validación de firma. Firma incompatible → `StubError::SignatureMismatch`.
6. **Casos negativos (7 mínimos)**: YAML inválido, op inexistente, import no autorizado, stub faltante, firma incompatible, trap del host, resultado incorrecto. Cada uno con test dedicado en `executor_yaml.rs`.
7. **Aislamiento entre cases**: test adversarial donde un case escribe en `host_state` y el siguiente lo verifica como estado inicial (fallaría si el estado se compartiera).
8. **Comparación `expected_host_calls`**: orden Y args verificados; un test lo contrasta.
9. **`SuiteReport` serializable** a JSON (`serde_json`).
10. **`poc-equivalence-matrix.md`** publicado con las 10 filas de pruebas unit POC `runner`, cada una con status explícito.
11. **Dependencia YAML**: `serde_yaml2` añadida como dep normal del runtime; validada contra fixtures POC. Si falla, se presenta alternativa antes de añadirla.
12. **Lint ADR-002/001**: `cargo tree -p agent-native-runtime --edges normal` sigue sin `agent-native-compiler`; `agent-native-ir` sigue sin WASM/sha2.
13. **CI remota verde 3 OS.** `cargo fmt`, `clippy pedantic -D warnings`.
14. **INIT-03-03 marcado `completed`** → **INI-003 cerrado 6/6**. Hito M3 **sujeto a aprobación explícita del usuario antes de ser declarado alcanzado**.

## Out of scope

- CLI `agent-native test <suite.yaml>` → INIT-05-01.
- Host functions reales que ejecuten lógica de dominio → INIT-06-01 / INIT-07-01.
- Evolución integral v1→v2 (esclusas) → M4 (INIT-04-03/04-05).
- Resources, strings, listas en WIT → RM-07.
- Soporte stub beyond StateReader/StateWriter/Noop (p. ej. error injection, delays) → futuro.
- Replicar los 338 LOC de `register_component_imports` del POC → scope infinito; stubs declarativos es el reemplazo pragmático.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p agent-native-runtime --test executor_yaml
cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler || echo OK
```

## Definition of Done

- 7 submodulos del módulo `testing` operativos.
- `Component::invoke_with_host_ctx` operativo, 43 tests previos verdes.
- ≥10 cases YAML + 7 adversariales verdes.
- Matriz POC publicada con 10 filas.
- CI remota verde 3 OS.
- WI listo para `kaddo learn` **tras aprobación humana de M3**.
- Entregables presentados: AC por criterio, matriz POC, APIs públicas, errores negativos, verificaciones workspace.

## Open questions residuales

- **OQ-AN-019-05 (nuevo)**: ¿`invoke_with_host_ctx` acepta Core WASM y Component Model? Propuesta: ambos; mismo pattern `wasmtime::Linker<T>` para Core; `wasmtime::component::Linker<T>` para Component Model.
- **OQ-AN-019-06 (nuevo)**: ¿el `host_state` del YAML permite tipos no-i32 (strings, maps anidados)? Propuesta: solo `i32` en MVP; otros tipos quedan `[open]` para INIT-07-02.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-03 — Executor de aceptación YAML |
| Hito | **M3 — Runtime autónomo con regresión POC** (candidato de declaración al cerrar este WI, sujeto a aprobación) |
| ADRs | ADR-002 (runtime autónomo), ADR-004 (fail-explicit + stubs no autorizan), ADR-005 (fresh Store per invoke) |
| Prerrequisitos | AN-009, AN-010, AN-011, AN-013, AN-014, AN-015 |
| Fuera de alcance | INIT-05-01 (CLI), INIT-06-01 (policy loader + host provider completo), INIT-04-03/05 (esclusas v1→v2 integral), RM-07 (WASI) |
| Evidencia pre-ready | Experimento de viabilidad del 2026-10-18: componente Core WASM v0.3 con `op: call_import` + Linker::func_wrap sobre `Store<HostCtx>`, verificado end-to-end (resultado correcto + call registrada en orden) |

## Learning

**Lo que salió como se esperaba:**

- La probe de viabilidad del 2026-10-18 fue predictiva: `wasmtime::Linker::func_wrap` + `Store<HostCtx>` acoplan sin fricción con el pre-check de policy ya existente. Las closures `Send+Sync` acceden al estado mutable vía `caller.data_mut()` sin `Rc`/`RefCell`.
- El extension point (`invoke_core_with_host_ctx` + `invoke_component_with_host_ctx`) se materializó sin duplicar lógica: `precheck_component_imports` y `precheck_core_imports` son los únicos puntos de autorización, y tanto `invoke` como las variantes `*_with_host_ctx` los invocan. `Component::invoke` no cambió de firma; los 60+ tests previos (smoke, resolver, policy, wit_policy, host_errors, nested_records, e2e_v05) siguen verdes sin tocarse.
- La separación "autorización ≠ registro de stubs" (ADR-004) se preserva con dos vías explícitas: `UnauthorizedByPolicy` corta la suite entera cuando se declara un stub sin autorización en policy; `UnauthorizedImport` corta por caso cuando la IR declara un import que la policy niega. Ambos tienen test adversarial.
- `serde_yaml2 = "0.1"` (OQ-01) fue la primera elección y validó contra el fixture smoke POC antes de integrarse como dep normal del runtime. No hubo que caer en `serde_yml` ni en features flags: módulo público + deps normales, decisión más simple y CI implícito.
- El schema YAML extendido con `host_stubs:` no toca ningún campo heredado del POC (`id`, `requirement`, `function`, `args`, `host_state`, `expected`, `expected_host_calls`). La adaptación es mínima (OQ-02 cumplido).
- El aislamiento entre casos cae gratis del diseño: `HostCtx` fresh por case (kv clonado de `host_state`, calls=vec![]) + fresh `Store` por invoke (ADR-005). El test `isolation.yaml` lo verifica como invariante observable (caso B lee lo que escribió el caso A → 0, no 777).

**Lo inesperado:**

- **Clippy pedantic `doc_markdown`** se quejó de "HostPolicy" sin backticks en 2 docstrings (`errors.rs` L23, `executor.rs` L82). Fix mecánico, pero vale la nota: la convención workspace-wide es poner backticks alrededor de identificadores Rust en comentarios de doc.
- **`clippy::needless_pass_by_value`** sobre `load_core(policy: HostPolicy)` en el test binario — `HostPolicy` es `Clone` pero el helper no lo consume, así que lo tomó por valor sin razón. Pasado por `&HostPolicy` y resuelto.
- **IR v0.5 requiere `wit_contract`**: el primer intento de test adversarial "payload Component Model rechazado" construyó una IR v0.5 inline sin `wit_contract` y falló en el verificador antes de llegar al runtime. Resuelto cambiando el test a ejercitar directamente `invoke_component_with_host_ctx` sobre un payload Core, que es la aserción simétrica que importaba.
- **Soporte Component-Model stubs diferido**: `run_component_case` existe como ruta explícita que devuelve `HostError("Component-Model host stubs are not yet wired (see RM-07)")`. El pre-check de autorización sigue corriendo igual, así que una suite dirigida a Component Model hoy se rechaza limpiamente. Más honesto que fingir soporte parcial.
- **Capability namespace libre**: el verificador IR no impone un vocabulario fijo para `capabilities` — basta con que la import referencie una capability declarada en root. Esto hizo trivial construir las IR factories del test binario (`kv_store`, `ambient`) sin negociar con el verificador.

**Riesgos materializados:**

- R-E-01 (closures `Send+Sync`): **mitigado por diseño**. `HostCtx` vive en `Store<T>::data`, nunca en captures.
- R-E-02 (duplicar lógica de `invoke.rs`): **mitigado**. Los dos helpers `precheck_*_imports` son el único sitio de autorización; invoke / invoke_*_with_host_ctx los comparten.
- R-E-03 (fixtures POC que requieren el emisor del POC): **no materializado en este alcance**. El scope MVP se queda en Core WASM + imports i32→i32; las fixtures POC más ricas (records, LLM-domain) quedan en `out-of-scope-M3` de la matriz de equivalencia.

**Deuda y seguimientos:**

- **Component Model host stubs**: `run_component_case` devuelve error tipado hoy. Enganche real → RM-07 (requiere mapear WIT import signatures a stubs, probablemente con una tabla de kind-por-WIT-type adicional).
- **`host_state` solo i32** (OQ-06): tipos ricos (strings, maps anidados) → INIT-07-02.
- **CLI `agent-native test <suite.yaml>`**: fuera de scope por diseño → INIT-05-01.
- **Esclusas v1→v2 integrales**: diferidas a M4 (INIT-04-03/04-05); la matriz de equivalencia marca test 7 como out-of-scope-M3.
- **Composición multi-componente**: diferida a RM-07; matriz marca test 5 como out-of-scope-M3.
- **Error injection + delays en stubs**: no comprometidos para M3; propuesta de extensión para cuando aparezca el primer caso real que los necesite.

**Lo que esta iniciativa establece:**

- **Agent-Native tiene regresión POC**: la matriz 10-filas publica status explícito por cada test del `runner/src/main.rs` del POC. 6 cubiertos (3 ported + 3 superseded), 4 explícitamente diferidos con destino de roadmap.
- **INIT-03-03 cumplido** desde la perspectiva de los AC del roadmap: `cargo test -p agent-native-runtime --test executor_yaml` corre 17 pruebas (11 cases happy + 8 negativas + 2 structurales, 5 suites). La condición "10 pruebas runner POC verdes" se interpreta como "cobertura equivalente documentada en matriz 1:1", no "ejecutar los 10 tests literales del POC".
- **M3 — Runtime autónomo con regresión POC** tiene su gating técnico cumplido; la **declaración formal del hito queda pendiente de aprobación humana explícita** (AC #14).
- **Patrón `invoke_with_host_ctx`** generalizable: cualquier WI futuro que necesite inyectar closures (fuel limiter, trace recorder, LLM host provider en INIT-06-01) reutiliza el mismo extension point. No habrá un segundo mecanismo paralelo.
- **Deny-by-default sin ambigüedad**: una stub declaration ya no puede, por error de redacción o copy-paste, abrir una capacidad; policy sigue siendo la única fuente de verdad. Esto robustece el modelo de amenaza antes de que llegue INIT-06-01 (policy loader + capability tokens).

### Evidencias de ejecución

- Commit implementación: `5b98d81` (feat(AN-019)) → push a `origin/main` (`14729f3..5b98d81`). CI remota [#39174154269](https://github.com/Kaddo-kdd/agent-native/actions/runs/39174154269) **success** 3/3 OS en 5m25s.
- Local Windows: `cargo test --workspace` → 60+ tests, 0 fallos (incluye los 17 nuevos de `executor_yaml`).
- Local Windows: `cargo clippy --workspace --all-targets -- -D warnings` → clean (0 warnings).
- Local Windows: `cargo fmt --all -- --check` → clean.
- Local Windows: `cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler` → exit 1 (ADR-002 preservado: runtime no depende de agent-native-compiler).
- Fixtures YAML: 5 suites, 11 cases, 4 con `host_stubs` (`smoke.yaml`, `kv-read.yaml`, `kv-write.yaml`, `noop.yaml`, `isolation.yaml`). AC #4 cumplido.
- Negative tests: 8 (invalid YAML, OperationNotFound, UnauthorizedImport, UnauthorizedByPolicy, StubMissing, SignatureMismatch, wrong-result Fail, cross-payload). AC #6 cumplido.
- Matriz POC: `knowledge/tech/poc-equivalence-matrix.md` con 10 filas, status explícito. AC #10 cumplido.

────────────────────────
Agent: work-item-agent
Produced:
  knowledge/delivery/work-items/completed/AN-019-executor-yaml.md
  crates/runtime/src/testing/* (7 submódulos)
  crates/runtime/src/{engine,invoke,lib}.rs (edits mínimos)
  crates/runtime/Cargo.toml (serde + serde_yaml2)
  crates/runtime/tests/executor_yaml.rs
  crates/runtime/tests/suites/*.yaml (5 ficheros)
  knowledge/tech/poc-equivalence-matrix.md
  Cargo.lock

Pendiente de autorización humana explícita (fuera de este cierre):
  1. Declaración formal del hito M3.
  2. Ejecución de `kaddo learn` (si procede como paso adicional).
────────────────────────
