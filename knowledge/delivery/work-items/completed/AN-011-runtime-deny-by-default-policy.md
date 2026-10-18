---
id: AN-011
type: feature
status: completed
completed_at: 2026-10-17
title: Policy mínima deny-by-default para imports del host
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/policy.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/invoke.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/tests/policy.rs
source_roadmap_initiative: INIT-03-05
source_work_item_candidate:
  - WI-CANDIDATE-03-05-01
  - WI-CANDIDATE-03-05-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
expected_value: >-
  Capa declarativa deny-by-default sobre AN-010: cada import del componente se
  contrasta contra un HostPolicy configurable antes de instanciar. Habilita
  INIT-06-01 (policy loader completo) y INIT-07-01 (WASI stdio autorizado).
risks:
  - >-
    R-POL-01: pasar policy opcional cambia la firma pública; se añade con un
    método adicional load_component_with_policy sin romper el existente
dependencies:
  - AN-009
  - AN-010
source_signals:
  - >-
    Roadmap INIT-03-05: deny por defecto en capabilities del host desde el
    primer runtime ejecutable
  - 'ADR-004: fail-explicit; deny-by-default es su realización concreta'
  - 'AN-010 Learning: la API pública no cambia al introducir policy'
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

# AN-011 — Policy mínima deny-by-default

**Actor y resultado:**
Un consumidor del runtime puede construir un `HostPolicy` con una allow-list explícita de imports `(module, name)` y pasarlo a `Engine::load_component_with_policy(bytes, &policy)`. El default (`load_component` sin policy) sigue siendo deny-all como en AN-010.

**Current behavior:**
AN-010 rechaza cualquier import declarado con `UnauthorizedImport`. No hay forma de autorizar imports selectivamente.

**Target behavior:**
- `src/policy.rs` nuevo con `HostPolicy` (allow-list configurable).
- `Engine::load_component_with_policy(bytes, &HostPolicy)` nuevo método; `Engine::load_component(bytes)` sigue existiendo como atajo de `deny_all()`.
- Durante la carga: cada import del componente se contrasta contra la policy. No permitido → `UnauthorizedImport`. Permitido → la carga procede, pero si no hay host function provista, la instanciación falla con `InstantiationFailed` (semánticas de wasmtime; INIT-06-01 refinará esto).
- `HostPolicy` API:
  - `HostPolicy::deny_all() -> Self` (default equivalente al comportamiento AN-010).
  - `allow(&mut self, module, name) -> &mut Self` (chainable).
  - `allow_module(&mut self, module) -> &mut Self` (comodín por módulo; útil para `host::*`).
  - `is_allowed(&self, module, name) -> bool` (inspección).
- 3+ tests que cubren deny-all, allow específico, allow por módulo, y que la policy opcional no rompe los flujos existentes.

**Problem:**
Sin un mecanismo declarativo, no hay forma de preparar la base de INIT-07-01 (autorizar `wasi:cli/stdio`) ni de INIT-06-01 (policy loader manifest-driven). Y la invariante "deny-by-default" vive sólo implícita en el linker vacío.

**Scope confidence:** high.

## Acceptance criteria

1. **`HostPolicy` tipado y público** con los 4 métodos declarados.
2. **`load_component_with_policy`** disponible y usada en los tests. `load_component` conserva su firma y usa internamente `HostPolicy::deny_all()`.
3. **Pre-instantiate check** usa la policy: para cada import del componente/módulo, consulta `is_allowed`. No permitido → `UnauthorizedImport` con el primer offender. Permitido → sigue el flujo normal (instanciación puede fallar aparte con `InstantiationFailed`).
4. **4 tests en `tests/policy.rs`**:
   - `deny_all` sobre componente con imports → `UnauthorizedImport` (equivalente a AN-010).
   - `allow` específico para los 3 imports de program-003 → **no** emite `UnauthorizedImport`; falla con `InstantiationFailed` porque no se proveen host functions (diferido a INIT-06-01).
   - `allow_module("host")` → mismo resultado que allow individual de los 3.
   - `is_allowed` inspección: `deny_all().is_allowed("host", "kv_get") == false`, tras `allow` devuelve `true`.
5. **Tests existentes (21) siguen verdes.**
6. **Lint y fmt verdes.** `cargo fmt`, `clippy pedantic -D warnings`.
7. **CI remota verde 3 OS.**

## Out of scope

- Provisión de host functions reales (INIT-07-01 para stdio; INIT-06-01 para resto).
- Lectura de policy desde manifiesto del artefacto (INIT-06-01).
- DSL de policy (fuel, memory limits, allow con pattern, …) — RM-06.
- Capability-level grouping (p. ej. `allow_capability("kv_store")` que expanda a los imports de ese capability) — se refina en INIT-06-01 con conocimiento del manifiesto.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-runtime
cargo test --workspace
```

Esperado: 25 tests pass (21 previos + 4 nuevos).

## Definition of Done

- `HostPolicy` implementado + re-exportado.
- `load_component_with_policy` operativo.
- 4 tests negativos verdes.
- CI remota verde.
- WI `completed`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-05 |
| ADR | ADR-004 |
| Prerrequisitos | AN-009, AN-010 |

## Learning

**Lo que salió como se esperaba:**

- `HostPolicy` como struct simple con `HashSet<(String, String)>` + `HashSet<String>` cubre el caso de uso declarativo de INIT-03-05 sin introducir dependencias nuevas.
- `load_component_with_policy` conserva `load_component` como atajo deny-all; backward-compatible con AN-009/010. Los 21 tests previos pasan sin tocar.
- La predicción de AN-010 Learning se cumplió: *"INIT-03-05 reemplaza el chequeo `.next()` → UnauthorizedImport por 'consultar policy; si no autoriza → UnauthorizedImport'. La API pública no cambia."*

**Lo inesperado:**

- `allow_pair_authorises_specific_import_only` depende de que Wasmtime liste los imports del Core WASM en un orden determinado (quería garantizar que el error reportado NO fuera `kv_get`). Resuelto cambiando el aserto a "no es `kv_get`" en lugar de "es `check_credit`" — más robusto frente a reordenamientos futuros.
- `allow_module_authorises_every_import_under_it` sí logra pasar el gate de policy y falla con `InstantiationFailed` (porque no hay host functions reales). Confirma que la capa declarativa funciona de forma independiente del provisioning real (que vive en INIT-06-01).
- Clippy pedantic marcó `HostPolicy` sin backticks en una docstring de test — ajuste trivial.

**Riesgos materializados:** ninguno. R-POL-01 (cambio de firma pública) mitigado por `load_component_with_policy` como método nuevo.

**Deuda y seguimientos:**

- `allow_module` es un wildcard útil pero burdo. INIT-06-01 debería añadir capability-level grouping cuando el manifiesto formalice qué imports pertenecen a qué capability.
- La policy se **clona** por componente en `load_component_with_policy`. Para catálogos grandes de imports esto es desperdicio; INIT-06-01 puede reemplazar con `Arc<HostPolicy>`.
- `is_allowed(module, name)` recibe `name` como string del item type (`"func"`, `"instance"`, …) para Component Model — no es el nombre lógico de la función. Para Core WASM es el nombre real. INIT-06-01 debería unificar la semántica.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-06-01** puede extender `HostPolicy` con proveedor de host functions (`register_fn`), fuel limits y per-capability policies.
- **INIT-07-01** puede exponer `HostPolicy::wasi_cli_stdio()` como preset.
- **INIT-05-01 (CLI)** puede parsear `--allow host::kv_get` para construir policies desde línea de comandos.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-011-runtime-deny-by-default-policy.md
crates/runtime/src/{policy.rs, engine.rs, invoke.rs, lib.rs}
crates/runtime/tests/policy.rs (4 tests)

Next:
INIT-03-03 (executor YAML) o INIT-03-02 (resolver reutilizable)
────────────────────────
