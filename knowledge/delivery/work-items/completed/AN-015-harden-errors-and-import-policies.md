---
id: AN-015
type: feature
status: completed
completed_at: 2026-10-17
title: Endurecimiento de errores y políticas por operación WIT
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/error.rs
  - crates/runtime/src/policy.rs
  - crates/runtime/tests/host_errors.rs
  - crates/runtime/tests/policy.rs
  - crates/runtime/tests/wit_policy.rs
source_roadmap_initiative: INIT-03-04
cross_initiatives:
  - INIT-03-05
source_work_item_candidate:
  - Technical Review 2026-10-17 (ref 06d270d) — WI-B
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Independent Runtime Execution
  - Host Capability Policy
expected_value: >-
  Elimina el fallback silencioso a Val::S32(0) y hace la policy útil para
  Component Model autorizando por **interfaz WIT real** y por **operación**,
  usando los identificadores que Wasmtime realmente expone (p. ej.
  `an:test/types@0.1.0`), nunca etiquetas genéricas (`"func"`, `"instance"`).
  Separa claramente **identificación / autorización / resolución**.
risks:
  - >-
    R-B-01: cambios en la API pública de HostPolicy pueden romper
    AN-010/011/014; mitigado añadiendo métodos nuevos y manteniendo
    allow()/allow_module() con semánticas documentadas
  - >-
    R-B-02: iterar operaciones de un ComponentInstance import puede ser costoso
    para interfaces grandes; mitigado con early-return en el primer unauthorized
dependencies:
  - AN-010
  - AN-011
  - AN-014
source_signals:
  - 'Technical Review: policy no distingue operaciones internas de una interfaz'
  - 'Technical Review: fallback silencioso Val::S32(0) viola ADR-004'
  - >-
    AN-014 Learning: `use types.{...}` genera un import `ComponentInstance`
    indistinguible de host-service externo
  - >-
    Probe exploratorio (examples/an015_probe_use_types.rs): confirma
    `an:probe/types@0.1.0` como import kind=ComponentInstance
  - >-
    crates/runtime/src/invoke.rs:49-50 is_allowed consulta item_label literal
    'func'
  - 'crates/runtime/src/invoke.rs:253 fallback S32(0) confirmado'
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

# AN-015 — Endurecer errores y políticas por operación WIT

**Actor y resultado:**
El runtime nunca mapea un valor desconocido a `Val::S32(0)` ni similar. Toda referencia WIT (import o export) se identifica por su nombre real (`an:test/ops@0.1.0`, `make-pt`, …), nunca por item kind. La `HostPolicy` autoriza explícitamente **por interfaz** (toda la interfaz) o **por operación** (una sola operación dentro de la interfaz), usando esos identificadores reales. Deny-by-default permanece intacto. `allow_module` sigue disponible por compatibilidad pero su alcance permisivo se documenta y se prueba.

## Precisiones arquitectónicas (incorporadas en el refinamiento)

Estas 7 reglas rigen el diseño y las pruebas:

1. **Las referencias WIT de tipos son imports como cualquier otro.** El runtime **no** autoriza automáticamente los imports porque pertenezcan al mismo package del componente. No existe atajo "mismo paquete = confiable". El llamador decide y declara.
2. **Identificación / autorización / resolución son 3 conceptos distintos:**
   - **Identificación:** qué nombre expone Wasmtime (`component_type().imports(engine)` → `(String, ComponentItem)`).
   - **Autorización:** si la `HostPolicy` lo permite.
   - **Resolución:** si el host provee una implementación efectiva (fuera de este WI; INIT-06-01/07-01).
   El invocador NO infiere resolución a partir de autorización ni viceversa.
3. **Políticas por interfaz Y por operación, con identificadores reales.** La API acepta:
   - `allow_wit_interface(interface_id)`: autoriza la interfaz completa (todas sus operaciones).
   - `allow_wit_operation(interface_id, operation)`: autoriza una operación específica dentro de la interfaz.
   - `allow_core(module, name)`: autoriza un import de Core WASM.
   Jamás se almacenan ni se consultan etiquetas genéricas como `"func"` o `"instance"`.
4. **Deny-by-default se preserva.** `HostPolicy::deny_all()` sigue siendo el default. `allow_module(module)` permanece como método disponible, pero su docstring declara explícitamente que **autoriza cualquier nombre bajo ese módulo** y que debe usarse con cuidado (preferir `allow_core` / `allow_wit_*` granulares). Una prueba demuestra su alcance permisivo.
5. **Fallback `Val::S32(0)` eliminado.** `core_results_to_component` cambia su firma a `Result<Vec<Val>, RuntimeError>`; cualquier tipo core distinto de `I32` emite `RuntimeError::ExecutionFailed { ... "unsupported core result type ..." }`. Las rutas válidas (I32 → S32) siguen funcionando: `smoke`, `host_errors`, `poc_regression` siguen verdes sin tocar aserts.
6. **Pruebas positivas y negativas** para:
   - Core WASM: `allow_core("host", "kv_get")` → otras ops `host.*` siguen bloqueadas.
   - Core WASM: `allow_module("host")` → todas pasan (regression para precisión 4).
   - Component Model interfaz-wide: `allow_wit_interface("an:test/ops@0.1.0")` → todas las operaciones OK.
   - Component Model op-wide: `allow_wit_operation("an:test/types@0.1.0", "pt")` → `pt` autorizado, otras siguen bloqueadas (fixture del probe reutilizado).
   - `use types.{...}` sin autorización → `UnauthorizedImport` con nombre real `"an:test/types@0.1.0"`.
   - Fallback core result inesperado (expresado por fixture hand-crafted si posible, en su defecto cubierto por el type system del enum sin rama default) → documentado.
7. **No proveedor completo de host ni WASI.** Autorizar `("an:test/types@0.1.0", "pt")` **no** proporciona una implementación. La instanciación seguirá fallando con `InstantiationFailed` hasta que INIT-06-01/07-01 lleguen. Este WI solo refina **autorización**, no **resolución**.

## Current behavior (verificado)

- `invoke.rs:49-50` (`invoke_component`): `policy.is_allowed(name, item_label)` donde `name` = interface identifier real (p. ej. `"an:test/types@0.1.0"`) y `item_label` = literal `"func"` o `"instance"`. Resultado: `allow("an:test/types@0.1.0", "pt")` nunca autoriza nada en Component Model porque el runtime compara con `item_label="instance"`.
- `invoke.rs:253` (`core_results_to_component`): `_ => Val::S32(0)` para cualquier `wasmtime::Val` que no sea `I32`. Comentario dice "unreachable under IR v0.5" pero el fallback existe.
- Core WASM funciona bien (comparación con `(module, name)` reales).

## Target behavior

### API `HostPolicy`

Métodos nuevos (añadidos sin romper los existentes):

```rust
impl HostPolicy {
    pub fn allow_core(&mut self, module: impl Into<String>, name: impl Into<String>) -> &mut Self;
    pub fn allow_wit_interface(&mut self, interface_id: impl Into<String>) -> &mut Self;
    pub fn allow_wit_operation(&mut self, interface_id: impl Into<String>, operation: impl Into<String>) -> &mut Self;
}
```

Compatibilidad:
- `allow(module, name)` sigue existiendo. Para Core WASM se comporta como `allow_core(module, name)`. Para Component Model (name arbitrario), se registra en el store Core (no de operación WIT); si el llamador quiere autorizar una operación WIT, debe usar `allow_wit_operation`. Docstring de `allow` actualizado.
- `allow_module(m)` sigue permitiendo CUALQUIER nombre bajo `m` para ambos modelos. Docstring actualizado con el aviso de permisividad.

Internamente la policy almacena 4 sets:
- `allow_core_pairs: HashSet<(String, String)>`
- `allow_modules: HashSet<String>` (comodín global, cross-model)
- `allow_wit_interfaces: HashSet<String>`
- `allow_wit_operations: HashSet<(String, String)>` (interface, operation)

### `invoke_component`

Para cada import del `component.component_type().imports(engine)`:
- Si `item` es `ComponentInstance(inst_ty)`:
  - Si `allow_wit_interface(name)` → autorizado, continuar.
  - Si `allow_module(name)` → autorizado, continuar.
  - En otro caso, iterar `inst_ty.exports(engine)` y para cada `(op_name, op_item)` consultar `allow_wit_operation(name, op_name)` o `allow_module(name)`. Si alguna op no está autorizada, emitir `UnauthorizedImport { module: name, name: op_name }`.
- Si `item` es `ComponentFunc(_)` top-level: consultar `allow_wit_operation(parent_module, name)` o fallback a `allow_module`. (Caso raro; identificador parent_module="" si no aplica.)
- Si no está autorizado por ninguna vía → `UnauthorizedImport` con `name` = interface id, `name` = operación o item name reales.

Jamás se pasa `"func"`/`"instance"` como identificador a `is_allowed`.

### `invoke_core`

Sin cambios en semántica. Internamente, `module.imports()` sigue dando `(module, name)` reales; `policy.is_allowed_core(module, name)` consulta `allow_core_pairs ∪ allow_modules`.

### `core_results_to_component`

```rust
fn core_results_to_component(results: &[wasmtime::Val]) -> Result<Vec<Val>, RuntimeError> {
    results.iter().map(|v| match v {
        wasmtime::Val::I32(i) => Ok(Val::S32(*i)),
        other => Err(RuntimeError::ExecutionFailed(format!(
            "unsupported core result type: {other:?}"
        ))),
    }).collect()
}
```

Caller en `invoke_core` encadena con `?`.

## Acceptance criteria

1. **Sin fallback silencioso.** No existe en el runtime ningún arm que mapee un `wasmtime::Val` desconocido a un `Val` por defecto. `core_results_to_component` ahora retorna `Result`.
2. **Policy consulta identificadores reales.** `is_allowed_component` (o equivalente) nunca recibe `"func"` ni `"instance"`.
3. **Granularidad por operación.** `allow_wit_operation("an:probe/types@0.1.0", "pt")` autoriza `pt` y nada más. Autorizar otra operación de la misma interfaz la deja aún bloqueada.
4. **Granularidad por interfaz.** `allow_wit_interface("an:probe/types@0.1.0")` autoriza TODAS las operaciones de esa interfaz.
5. **`use types.{...}` fixture.** Fixture del probe (`an:probe` package con `use types.{pt}`) ejercita el camino: sin policy → `UnauthorizedImport { module: "an:probe/types@0.1.0", name: <op real> }`; con `allow_wit_interface` → pasa a instancia (y falla con `InstantiationFailed`, por diseño de este WI).
6. **`allow_module` documentado y probado.** Docstring con advertencia; test demuestra que autoriza cualquier operación bajo ese módulo.
7. **Deny-by-default intacto.** `HostPolicy::deny_all()` sigue siendo default. Un componente con imports y sin policy explícita devuelve `UnauthorizedImport`.
8. **Core WASM:** tests independientes para `allow_core` y `allow_module` sobre Core WASM.
9. **Tests existentes (33) siguen verdes** (adaptaciones mínimas de aserts si antes aceptaban `name = "func"` → ahora el nombre real; AN-011 `allow_pair_authorises_specific_import_only` ya usa `allow("host", "kv_get")` real, no hay cambio esperado ahí; AN-010 `core_component_with_declared_import_is_unauthorized` igualmente).
10. **Lint y fmt verdes.** `cargo fmt`, `clippy pedantic -D warnings`.
11. **CI remota verde** 3 OS.

## Out of scope

- Implementación completa de WASI (INIT-07-01).
- Proveedor real de host functions (INIT-06-01).
- Motor de políticas multi-tenant (INIT-06-03).
- Gestión de secretos / credenciales (futuro).
- Pattern matching en policy (`allow("wit:*", "op_a")`).
- Métricas/observabilidad de denegaciones.
- Mecanismos de "same-package auto-allow" — explícitamente **rechazado** por precisión 1.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-runtime
cargo test --workspace
```

Esperado: ≥38 tests pass (33 + 5+ nuevos de WIT policy).

## Definition of Done

- Fallback silencioso eliminado.
- Policy con `allow_wit_interface`/`allow_wit_operation`/`allow_core` operativos y documentados.
- Tests positivos y negativos verdes.
- Fixture `use types.{...}` ejercitada.
- CI remota verde 3 OS.
- WI `completed` (sujeto a aprobación humana explícita).

## Open questions

- OQ-AN-015-01: ¿`allow_module` debe deprecarse formalmente con `#[deprecated]`? Propuesta: NO — se queda como atajo documentado con advertencia; deprecar es cambio potencialmente breaking que merece su propio WI.
- OQ-AN-015-02: ¿`allow(module, name)` existente se interpreta como `allow_core` por defecto o se marca deprecated también? Propuesta: redirigir internamente a `allow_core` y actualizar docstring. Pruebas de AN-010/011 que usan `allow("host", "kv_get")` siguen funcionando.
- OQ-AN-015-03: ¿Vale la pena un test que construya un Core WASM con retorno `I64` para forzar la rama de error de `core_results_to_component`? Propuesta: documentarlo como out-of-scope para este WI (requiere fixture hand-crafted WAT); la invariante queda cubierta a nivel de tipo con `Result<...>`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-04 (principal), INIT-03-05 (refuerza) |
| ADRs | ADR-004 |
| Technical Review | WI-B (ref 06d270d) |
| Prerrequisitos | AN-010, AN-011, AN-014 |
| Precisiones usuario | Mensaje autorización AN-015 (7 reglas) |

## Learning

**Lo que salió como se esperaba:**

- La separación en 4 sets internos de `HostPolicy` (`allow_core_pairs`, `allow_modules`, `allow_wit_interfaces`, `allow_wit_operations`) permite granularidad por operación y por interfaz sin romper las APIs previas. Las 33 pruebas de AN-010/011/014 pasaron intactas; el cambio quedó incremental.
- `invoke_component` walker: para cada `ComponentInstance` import se consulta primero interface-wide; si no está autorizado se iteran operaciones con early-return en la primera denegación. Simple, auditado por los tests.
- El fallback silencioso en `core_results_to_component` se convirtió en `Result<Vec<Val>, RuntimeError>` sin tocar los callers válidos (todos los retornos I32→S32 funcionan igual). La invariante queda en el tipo, no oculta.

**Lo inesperado (confirmado experimentalmente, no supuesto):**

- **Separación identificación / autorización / resolución (precisión 2):** los imports WIT **type-only** (como `use types.{pt}`) se resuelven **automáticamente** por Wasmtime una vez autorizados. No necesitan host function. Esto no significa que "imports del mismo paquete sean confiables" — significa que **para interfaces de solo tipos, resolución es un no-op**. Para function imports (host-services), la autorización no provee implementación; eso llega en INIT-06-01 / INIT-07-01.
- Confirmación con el fixture `probe-use-types`: autorizar `an:probe/types@0.1.0` ejecuta el componente end-to-end, no falla con `InstantiationFailed`.
- Los asserts iniciales de las pruebas asumían `InstantiationFailed`; se corrigieron tras observar la ejecución real. Esta corrección temprana es evidencia de disciplina "verificar, no asumir".

**Observación sobre Kaddo (sin modificar en este WI):**

- `kaddo verify AN-015 --json` reportó `"readiness": "READY_TO_COMPLETE"` desde el primer momento en `in-progress`, **antes** de que la CI remota corriera. El AC #11 ("CI remota verde 3 OS") es un criterio verificable solo por evidencia externa; kaddo no lo captura. Esto no cambió el flujo (yo esperé explícitamente la CI antes de `kaddo learn` como pediste), pero registra el hallazgo para una revisión posterior de Kaddo: la readiness de un WI con AC externos debería requerir evidencia registrada. **No modifico Kaddo en AN-015.**

**Riesgos materializados:** ninguno.
- R-B-01 (API breaking para AN-010/011): mitigado al mantener `allow`/`is_allowed`/`allow_module` como shims con semánticas documentadas.
- R-B-02 (coste del walker): no observable con fixtures de 1-3 operaciones; early-return mitiga.

**Deuda y OQs:**

- OQ-AN-015-01 (deprecar `allow_module`): resuelta en diseño — no se deprecar, se queda con advertencia explícita.
- OQ-AN-015-02 (interpretación de `allow(...)`): resuelta — redirige a `allow_core`.
- OQ-AN-015-03 (fixture WAT para rama de error core non-I32): sigue abierta, cubierta por el type system del enum.

**Lo que esta iniciativa establece:**

- **AN-016**: la convención de nombres snake_case (IR) vs kebab-case (WIT) y la convención de identificadores reales de Wasmtime (`an:pkg/iface@ver`) quedan documentados en código, test y docstring; AN-016 productivo puede referenciar estos nombres sin ambigüedad.
- **INIT-06-01** (policy loader completo): extiende `HostPolicy` con proveedores (`register_fn`) y lectura desde manifiesto, no cambia la taxonomía básica aquí definida.
- **INIT-07-01** (WASI stdio): puede exponer `HostPolicy::wasi_cli_stdio()` como preset de `allow_wit_interface("wasi:cli/stdio@...")`.
- **INIT-05-01** (CLI `run`): puede exponer `--allow-interface`, `--allow-operation`, `--allow-core` mapeando 1:1 a los métodos nuevos.

### Evidencias de ejecución

- Commit implementación: `20b23ff` → CI remota [`#39097746764`](https://github.com/Kaddo-kdd/agent-native/actions/runs/39097746764) **success** en ubuntu-latest, windows-latest, macos-latest.
- Local Windows: `cargo test --workspace` → 39 pruebas, 0 fallos.
- `kaddo verify AN-015 --json`: `"readiness": "READY_TO_COMPLETE"`, 0 blockers, 0 warnings, 0 AC `not-verified`.
- `cargo tree -p agent-native-runtime --edges normal | grep compiler` → vacío (ADR-002 intacto).

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-015-harden-errors-and-import-policies.md
crates/runtime/src/{policy,invoke}.rs
crates/compiler/tests/fixtures/{ir,wit}/probe-use-types.*
crates/runtime/tests/wit_policy.rs (6 tests)

Next (pending user authorisation):
AN-016 productivo con los hallazgos de knowledge/tech/discovery/memory-model-experiments.md,
o AN-017 (consolidación benchmarks/docs).
────────────────────────
