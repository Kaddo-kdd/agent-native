---
id: AN-045
type: feature
status: ready
created_at: 2026-10-20T00:00:00.000Z
title: Autorización granular de stdio y streams bajo Requested ∩ Granted
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/policy.rs
  - crates/runtime/src/manifest.rs
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/tenant.rs
  - crates/runtime/tests/wasi_policy.rs
source_roadmap_initiative: INIT-07-01
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
| `allow_wasi_stdout()` | `wasi:cli/stdout` + soporte `wasi:io/streams`, `wasi:io/error`, `wasi:io/poll` |
| `allow_wasi_stderr()` | `wasi:cli/stderr` + soporte `wasi:io/*` |
| `allow_wasi_stdin()` | `wasi:cli/stdin` + soporte `wasi:io/*` |
| `allow_wasi_stdio()` | los tres anteriores |

Garantías: ningún helper concede `wasi:filesystem/*`, `wasi:sockets/*`, `wasi:clocks/*`, `wasi:random/*`, `wasi:cli/environment` ni `wasi:cli/exit`.

### 2. Manifiesto: sin cambio de esquema (decisión propuesta)
Un componente solicita sus capacidades con los campos existentes (`wit_interfaces: ["wasi:cli/stdout@0.2.0", ...]`). **No se introduce `categories: [stdio]`** en el manifiesto 0.1: cambiaría el contrato `deny_unknown_fields` y obligaría a versionarlo. `stdio` queda como helper del lado host. Alternativa (descartada por defecto): manifiesto `0.2` con `categories`.

### 3. Enforzamiento fail-explicit
- El precheck existente rechaza imports no concedidos con `UnauthorizedImport { module: "wasi:cli/stdin@0.2.0", name: <operación|tipo> }`.
- Ningún código guest se ejecuta ante un rechazo.
- Un componente que importa solo `stdout` + `io/*` y es concedido solo `stdout` ejecuta; si importa además `stdin`, falla en precheck.

## Contratos Públicos Propuestos

```rust
impl HostPolicy {
    pub fn allow_wasi_stdout(&mut self) -> &mut Self;
    pub fn allow_wasi_stderr(&mut self) -> &mut Self;
    pub fn allow_wasi_stdin(&mut self) -> &mut Self;
    pub fn allow_wasi_stdio(&mut self) -> &mut Self;
}
```
Cada helper delega en `allow_wit_interface(..)` con los IDs de la tabla; ninguna lógica nueva de decisión en el motor.

## Acceptance Criteria
- **AC-01 (deny-by-default):** con `HostPolicy::deny_all()`, un componente que importa `wasi:cli/stdout` es rechazado con `UnauthorizedImport`.
- **AC-02 (independencia):** con `allow_wasi_stdout()`, un componente que además importa `wasi:cli/stdin` es rechazado pre-instantiate; el rechazo no ejecuta código guest (demostrable porque `fuel_consumed` no se reporta / fixture con `start` observable no corre).
- **AC-03 (aislamiento de otras áreas):** conceder `stdio` no habilita `wasi:filesystem/*`, `wasi:sockets/*`, `wasi:clocks/*`, `wasi:random/*`, `environment` ni `exit`; un componente que los importe es rechazado.
- **AC-04 (manifiesto):** un `ComponentManifest` 0.1 con `wit_interfaces` de WASI se parsea (YAML y JSON) y, aplicado a un `Tenant` cuya política concede solo stdout, la intersección permite stdout y rechaza el resto. Un manifiesto no puede autoconceder.
- **AC-05 (suite `wasi_policy.rs`):**
  1. stdout concedido → ejecuta y captura salida.
  2. stdout no concedido → falla pre-ejecución con error tipado.
  3. stdout concedido, componente importa stdin → falla con error tipado.
  4. `Tenant` con `allow_wasi_stdio()` vs `allow_wasi_stdout()` → comportamiento distinto y correcto por tenant.
  5. Inspección: `is_wit_interface_allowed("wasi:cli/stdin@0.2.0")` es `false` tras `allow_wasi_stdout()`.

## Decisiones aprobadas (INIT-07-01)
1. **Manifiesto v0.1 sin `categories`.** Los helpers del host facilitan la configuración sin modificar el formato declarativo.
2. **Helpers independientes:** `allow_wasi_stdout()` no autoriza `stdin` ni `stderr`; cada uno concede únicamente su interfaz y las auxiliares `wasi:io/*`. `allow_wasi_stdio()` es una conveniencia explícita que concede los tres.
3. **Versiones WIT: igualdad exacta.** `HostPolicy` y `ComponentManifest` comparan el identificador completo, incluida la versión. Que el `Linker` de Wasmtime resuelva `@0.2.x` compatibles no amplía la autorización: `wasi:cli/stdout@0.2.0` concedido y `@0.2.3` importado ⇒ rechazo explícito. Las pruebas de versión viven en el spike de AN-044 (AC-S1).
4. **`environment` y `exit` diferidos:** sin helpers ni implementaciones en esta etapa; se tratarán en la ampliación de binarios WASI reales.
5. **Interfaces auxiliares (`wasi:io/streams`, `error`, `poll`):** autorizarlas no es una vía indirecta para obtener un recurso denegado. El acceso real depende de la política **y** de los handles que entrega el contexto WASI: sin `wasi:cli/stdin` enlazado no existe handle de lectura.
6. **Orden:** AN-045 se implementa después de AN-044.

## Escenario obligatorio adicional
**AC-06 (auxiliares sin handle de entrada):** componente con `stdout` + `wasi:io/streams`/`error`/`poll` concedidos que importa también `wasi:cli/stdin` ⇒ rechazado pre-instantiate con `UnauthorizedImport` (módulo `wasi:cli/stdin@<versión>`), sin ejecutar código guest. Complementario: un componente que solo usa `stdout` y las auxiliares no obtiene ningún handle de `stdin` ni `stderr`.
**AC-07 (versiones):** el helper concede exactamente la versión exacta configurada; un import de otra versión de la misma interfaz se rechaza (coherente con AC-S1 de AN-044).

## Validation

- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-targets` y `cargo build --workspace --release` en verde.
- Suite `crates/runtime/tests/wasi_policy.rs` cubriendo AC-01..AC-07, incluido el escenario de auxiliares sin handle de `stdin` y las pruebas de igualdad exacta de versión.
- Revisión de que el esquema `ComponentManifest` 0.1 no cambia.
