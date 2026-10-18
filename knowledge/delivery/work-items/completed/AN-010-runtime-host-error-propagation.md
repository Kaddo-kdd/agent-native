---
id: AN-010
type: feature
status: completed
completed_at: 2026-10-17
title: Propagación tipada de errores host + imports no autorizados
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/error.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/tests/host_errors.rs
source_roadmap_initiative: INIT-03-04
source_work_item_candidate:
  - WI-CANDIDATE-03-04-01
  - WI-CANDIDATE-03-04-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  El runtime detecta imports no provistos por el host y errores de tipo de
  argumento emitiendo variantes tipadas de RuntimeError (nunca valor por
  defecto). Gate obligatorio de INIT-05-01 (CLI distribuible) y prerequisito de
  INIT-03-05 (policy loader deny-by-default).
risks:
  - >-
    R-ERR-01: Wasmtime no expone metadata estructurada de imports faltantes
    antes de instantiate; se detecta inspeccionando wasmparser/component types
    pre-link
dependencies:
  - AN-009
source_signals:
  - 'ADR-004: fail-explicit transversal, nunca fallback silencioso'
  - 'Roadmap principio #4: nunca producir 0/false ante error'
  - 'Matriz de extracción C-07: la POC ya hace esto; preservar invariante'
decision_candidates: []
related_decisions:
  - ADR-004
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-010 — Propagación tipada de errores host + imports no autorizados

**Actor y resultado:**
Un componente con imports host declarados (p. ej. IR v0.3+ con `imports`/`capabilities`) que no cuenta con host functions provistas produce `RuntimeError::UnauthorizedImport { module, name }` **antes** de ejecutar código del guest. Argumentos de tipo incorrecto producen `RuntimeError::ArgTypeMismatch`.

**Current behavior:**
AN-009 implementó `RuntimeError` con `LoadFailed/FunctionNotFound/ArityMismatch/InstantiationFailed/ExecutionFailed`. Cualquier componente con imports sin host actualmente falla con `InstantiationFailed(<wasmtime stringified>)` — tipado genérico, no accionable.

**Target behavior:**
- Nuevas variantes en `RuntimeError`:
  - `UnauthorizedImport { module: String, name: String }` — un import declarado por el componente no fue provisto por el host.
  - `ArgTypeMismatch { function: String, index: usize, expected: String, found: String }` — un argumento al invocar no coincide con la firma.
- `invoke` detecta imports faltantes **antes** de instanciar:
  - Para Core WASM: `wasmparser::parse_payload_all` extrae la lista de imports; si el `Linker<()>` vacío no los define, emite `UnauthorizedImport` con el primer import no listado.
  - Para Component Model: inspección de `wasmtime::component::types::ComponentItem::ComponentInstance` análoga.
- `invoke` valida tipos de args antes de llamar: si `args[i]` no coincide con `func.params(&store)[i]`, emite `ArgTypeMismatch`.
- 3+ pruebas negativas que cubren los 3 nuevos escenarios.

**Problem:**
Sin propagación tipada, cualquier consumidor del runtime (CLI, SDK, test harness futuro) tiene que parsear strings de Wasmtime para diferenciar causas. Rompe la invariante "fail-explicit".

**Scope confidence:** high. Mapea 1:1 a lo que la POC ya hace en `runner`.

## Acceptance criteria

1. **Nuevas variantes.** `RuntimeError` incluye `UnauthorizedImport` y `ArgTypeMismatch` con los campos descritos.
2. **Detección pre-instantiate** de imports no provistos: Core WASM y Component Model.
3. **Validación de args.** Si el tipo de `args[i]` no es compatible con `func.params(&store)[i]`, emite `ArgTypeMismatch` sin llamar a `func.call`.
4. **3 pruebas negativas** en `tests/host_errors.rs`:
   - Core WASM con import no satisfecho → `UnauthorizedImport { module, name }`.
   - Component Model con import no satisfecho → `UnauthorizedImport` (si el fixture lo permite; de lo contrario, documentado como diferido a INIT-03-05).
   - `identity(Bool)` cuando se espera `S32` → `ArgTypeMismatch`.
5. **Tests existentes siguen verdes** (18 anteriores).
6. **Lint y fmt verdes.** `cargo fmt`, `clippy -D warnings` workspace.
7. **CI remota verde** 3 OS.

## Out of scope

- Policy loader manifest-driven (INIT-03-05 / INIT-06-01).
- Allow-list configurable: aquí el "host registry" es siempre vacío; cualquier import falla. INIT-03-05 lo convierte en configurable.
- Suministro de host functions reales (p. ej. `kv_store`, `audit_log` del POC) — pertenece a RM-07 (WASI stdio) o a INIT-03-05.
- Fuel/instance limits (RM-06).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-runtime
cargo test --workspace
```

Esperado: 21 tests pass (18 previos + 3 nuevos).

## Definition of Done

- Variantes añadidas y propagadas en `invoke`.
- 3 pruebas negativas verdes.
- CI remota verde.
- WI `completed`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-04 |
| ADR | ADR-004 |
| Prerrequisito | AN-009 |

## Learning

**Lo que salió como se esperaba:**

- Las 2 variantes nuevas (`UnauthorizedImport`, `ArgTypeMismatch`) encajaron sin romper tests existentes.
- Detectar imports pre-instantiate con `module.imports().next()` (Core) y `component.component_type().imports(engine).next()` (Component Model) es trivial y auditablemente eficiente — no requiere parsing con `wasmparser`.
- Reutilizar `program-003.ir.json` como fixture para el escenario con imports evita escribir IR a mano (y errores de verifier).

**Lo inesperado:**

- La API de `wasmtime::component::Val` en 27.0 **no** tiene `Future/Stream/ErrorContext` que esperaba de versiones más nuevas. Lista completa descubierta tras compile error; documentada en `component_val_kind`.
- `wasmtime::component::types::Func::params()` devuelve `[Type]` directo, no `[(String, Type)]`. Ajuste trivial.
- El verifier de IR rechaza `op: const` sin campo `type` — primera iteración de IR a mano falló aquí. Lección: para pruebas, es más seguro usar fixtures ya validadas por POC que construir IR inline.

**Riesgos materializados:**

- R-ERR-01 nulo: Wasmtime sí expone metadata estructurada de imports vía `Module::imports()` y `Component::component_type().imports()`. El plan B (wasmparser) no fue necesario.

**Deuda y seguimientos:**

- `component_val_kind` lista 23 variantes; cuando Wasmtime bump a 28+ y aparezcan Future/Stream/ErrorContext, los arms faltantes hay que añadirlos. Un `#[allow(clippy::wildcard_enum_match_arm)]` evitaría romper CI pero oculta la deuda — prefiero el panic implícito actual.
- La validación de tipos es laxa: `component_val_matches_type` acepta primitivos solo. Cuando IR v0.6 introduzca `string` (RM-07 / INIT-07-02), extender.
- Detectar _cuál_ import específico de los varios declarados es responsable sería más preciso que reportar el primero; se refina cuando INIT-03-05 añada allow-list configurable.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-03-05 (deny-by-default)** reemplaza el chequeo "`.next()` → UnauthorizedImport" por "consultar policy; si no autoriza → UnauthorizedImport". La API pública no cambia.
- **INIT-05-01 (CLI)** puede mapear los 7 variantes a códigos de salida distintos.
- **INIT-06-01 (policy loader)** extiende este contrato con `MissingCapability` sin romper los existentes.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-010-runtime-host-error-propagation.md
crates/runtime/src/{error,invoke}.rs
crates/runtime/tests/host_errors.rs (3 tests)
crates/compiler/tests/fixtures/ir/program-003.ir.json

Next:
INIT-03-05 (allow-list manifest-driven) o INIT-03-03 (executor YAML)
────────────────────────
