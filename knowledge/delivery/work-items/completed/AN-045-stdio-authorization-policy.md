---
id: AN-045
type: feature
status: completed
created_at: 2026-10-20T00:00:00.000Z
completed_at: 2026-10-20T02:39:00.000Z
title: Autorización granular de stdio y streams bajo Requested ∩ Granted
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/policy.rs
  - crates/runtime/src/manifest.rs
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/tenant.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/tests/wasi_policy.rs
source_roadmap_initiative: INIT-07-01
initiative: INI-007
source_work_item_candidate:
  - WI-CANDIDATE-07-01-02
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
expected_value: >-
  Aplicar el modelo Requested ∩ Granted (ADR-004) y deny-by-default a las
  interfaces de I/O de WASI 0.2 (`wasi:cli/stdin`, `wasi:cli/stdout`,
  `wasi:cli/stderr` y soporte `wasi:io/*`), permitiendo conceder permisos de
  forma independiente (p. ej. stdout sin stdin, filesystem ni red) y rechazando
  con `RuntimeError::UnauthorizedImport` cualquier importación no autorizada
  antes de ejecutar código guest.
risks:
  - >-
    R-SEC-WASI-01: Auto-concesión implícita por registro de proveedores en el
    linker. Mitigación: el linker se puebla solo con interfaces concedidas
    (AN-044, AC-04); esta WI añade pruebas negativas end-to-end que lo
    demuestran.
  - >-
    R-SEC-WASI-02: Ambigüedad entre la macro-capability `stdio` y las interfaces
    WIT concretas. Mitigación: `stdio` existe solo como azúcar del host
    (`allow_wasi_stdio()`) que se expande a IDs WIT concretos; el motor solo
    razona con IDs WIT reales.
  - >-
    R-SEC-WASI-03: Un manifiesto es metadato no confiable (manifest.rs, RM-06).
    No puede conceder permisos. Mitigación: el manifiesto solo declara lo
    *solicitado*; lo concedido proviene siempre de `HostPolicy`.
  - >-
    R-SEC-WASI-04: Conceder `wasi:io/streams` por ser dependencia de stdout
    podría interpretarse como permiso de lectura. Mitigación:
    `streams`/`error`/`poll` son solo soporte de tipos; sin `stdin` enlazado no
    hay handle de lectura. Se cubre con test negativo.
dependencies:
  - AN-033
  - AN-034
  - AN-044
source_signals:
  - 'Roadmap RM-07: INIT-07-01 capability stdio autorizada'
  - 'ADR-004: Deny-by-default en capabilities del host y fail-explicit'
  - >-
    Decisión de Arquitectura RM-07: Permisos separados de I/O e interfaces WIT
    concretas
decision_candidates: []
related_decisions:
  - ADR-004
ready_at: '2026-10-20'
---

# AN-045 — Autorización granular de stdio y streams bajo Requested ∩ Granted

## Actor y Resultado
Un operador declara qué interfaces de I/O de WASI 0.2 concede a un componente. El runtime evalúa Requested ∩ Granted: si el componente importa `wasi:cli/stdout` y la política no lo concede, la invocación se rechaza con `RuntimeError::UnauthorizedImport` **antes de instanciar**, sin ejecutar código guest.

## Estado actual verificado (base de diseño)
- `precheck_component_imports` (`invoke.rs`) ya valida cada import de componente contra `HostPolicy` (interfaz u operación), pre-instantiate.
- `RuntimeError::UnauthorizedImport { module, name }` (el campo es `name`, no `function`).
- `ComponentManifest` (versión `0.1`, `deny_unknown_fields`) solo admite `wit_interfaces`, `wit_operations` y `core_imports`. **No existe** un campo `categories` ni `wit:`.

## Principios

### 1. Permisos separados
| Helper | Concede (IDs WIT concretos) |
|---|---|
| `allow_wasi_stdout()` | `wasi:cli/stdout@0.2.0` + soporte `wasi:io/streams@0.2.0`, `wasi:io/error@0.2.0`, `wasi:io/poll@0.2.0` |
| `allow_wasi_stderr()` | `wasi:cli/stderr@0.2.0` + soporte `wasi:io/*@0.2.0` |
| `allow_wasi_stdin()` | `wasi:cli/stdin@0.2.0` + soporte `wasi:io/*@0.2.0` |
| `allow_wasi_stdio()` | los tres anteriores |

Garantías: ningún helper concede `wasi:filesystem/*`, `wasi:sockets/*`, `wasi:clocks/*`, `wasi:random/*`, `wasi:cli/environment` ni `wasi:cli/exit`.

### 2. Manifiesto: sin cambio de esquema (decisión aprobada)
Un componente solicita sus capacidades con los campos existentes (`wit_interfaces: ["wasi:cli/stdout@0.2.0", ...]`). **No se introduce `categories: [stdio]`** en el manifiesto 0.1. `stdio` queda como helper del lado host.

### 3. Enforzamiento fail-explicit y defensa en profundidad
- El precheck existente rechaza imports no concedidos con `UnauthorizedImport { module: "wasi:cli/stdin@0.2.0", name: <operación|tipo> }`.
- Ningún código guest se ejecuta ante un rechazo (incluso si el componente contiene una función `(start ...)`).
- El `WasiCtx` solo conecta el stream `stdin`, `stdout` o `stderr` si la interfaz `wasi:cli/*` correspondiente fue efectivamente importada y autorizada; conceder `wasi:io/streams` jamás adjunta handles de entrada o salida no autorizados.

## Contratos Públicos Entregados

```rust
pub const WASI_CLI_STDIN_0_2_0: &str = "wasi:cli/stdin@0.2.0";
pub const WASI_CLI_STDOUT_0_2_0: &str = "wasi:cli/stdout@0.2.0";
pub const WASI_CLI_STDERR_0_2_0: &str = "wasi:cli/stderr@0.2.0";
pub const WASI_IO_STREAMS_0_2_0: &str = "wasi:io/streams@0.2.0";
pub const WASI_IO_ERROR_0_2_0: &str = "wasi:io/error@0.2.0";
pub const WASI_IO_POLL_0_2_0: &str = "wasi:io/poll@0.2.0";

impl HostPolicy {
    pub fn allow_wasi_stdout(&mut self) -> &mut Self;
    pub fn allow_wasi_stderr(&mut self) -> &mut Self;
    pub fn allow_wasi_stdin(&mut self) -> &mut Self;
    pub fn allow_wasi_stdio(&mut self) -> &mut Self;
}
```

## Verificación (AC-01..AC-07)
Cubiertos en `crates/runtime/tests/wasi_policy.rs`:
- `ac01_deny_by_default_rejects_wasi_stdout_pre_instantiate`: `HostPolicy::deny_all()` rechaza `wasi:cli/stdout@0.2.0`.
- `ac02_and_ac05_helper_independence_and_inspection`: verifica independencia estricta entre `allow_wasi_stdout()`, `allow_wasi_stderr()`, `allow_wasi_stdin()` y `allow_wasi_stdio()`.
- `ac02_and_ac06_stdout_and_aux_io_granted_still_rejects_stdin_pre_instantiate`: componente con `stdout` + `wasi:io/*` concedidos que además importa `wasi:cli/stdin@0.2.0` y tiene un `(start ...)` explosivo (`unreachable`) es rechazado pre-instantiate con `UnauthorizedImport` sin ejecutar `start`.
- `ac03_allow_wasi_stdio_never_grants_filesystem_sockets_clocks_random_env_or_exit`: verifica que `allow_wasi_stdio()` rechaza `filesystem`, `sockets`, `clocks`, `random`, `environment` y `exit`.
- `ac04_manifest_v01_intersection_and_no_categories_field`: verifica rechazo de `categories` en manifiesto v0.1 (`deny_unknown_fields`), equivalencia YAML/JSON y `Requested ∩ Granted` sin autoconcesión.
- `ac05_tenant_with_stdio_vs_tenant_with_stdout_only`: verifica aislamiento de políticas por `Tenant`.
- `ac07_helpers_enforce_exact_0_2_0_version_and_reject_other_versions`: verifica rechazo de `@0.2.3` y `@0.3.0` cuando se concede `@0.2.0`.
