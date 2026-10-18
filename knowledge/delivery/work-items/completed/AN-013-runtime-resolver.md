---
id: AN-013
type: feature
status: completed
completed_at: 2026-10-17
title: Resolver de componentes con caché content-addressed
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - crates/runtime/src/resolver.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/lib.rs
  - crates/runtime/tests/resolver.rs
  - crates/runtime/benches/baseline.rs
source_roadmap_initiative: INIT-03-02
source_work_item_candidate:
  - WI-CANDIDATE-03-02-01
  - WI-CANDIDATE-03-02-02
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Performance
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Caché content-addressed de componentes parsed/compiled. Reemplaza el patrón de
  la POC (recompilación por invocación) con un resolver reutilizable + reset por
  tenant. Mejora medible contra la línea base de AN-012.
risks:
  - >-
    R-RES-01: compartir Modules/Components entre tenants podría introducir fugas
    de estado si se guardan Stores; mitigado NO cacheando Stores — sólo los
    artefactos parseados
dependencies:
  - AN-009
  - AN-011
  - AN-012
source_signals:
  - 'Open Question L147: resolver reutilizable (POC recompila por invocación)'
  - 'AN-012 Learning: warm invoke 195× más rápido que cold — techo cuantificable'
  - 'Matriz C-07: resolver cacheado + reset por tenant'
decision_candidates: []
related_decisions:
  - ADR-001
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-013 — Resolver de componentes con caché content-addressed

**Actor y resultado:**
Un consumidor construye un `Resolver`, llama `resolver.load(bytes)` repetidamente con los mismos bytes y obtiene un `Component` reutilizable sin pagar el costo de parsing + Cranelift compile en cada llamada. `resolver.reset()` limpia la caché (ADR-005: aislamiento entre tenants).

**Current behavior:**
`Engine::load_component` parsea el WASM en cada invocación. AN-012 midió este costo en **~247 µs** (load_only). Cada carga repetida de los mismos bytes es desperdicio puro.

**Target behavior:**
- `src/resolver.rs` con `Resolver` que mantiene una caché `HashMap<[u8; 32], CachedPayload>` indexada por SHA-256 de los bytes de entrada.
- API:
  - `Resolver::new() -> Result<Self, RuntimeError>`
  - `Resolver::load(&mut self, bytes) -> Result<Component, RuntimeError>` (deny-all policy por default).
  - `Resolver::load_with_policy(&mut self, bytes, &HostPolicy) -> Result<Component, RuntimeError>`.
  - `Resolver::reset(&mut self)` — vacía la caché (uso: fin de sesión de tenant per ADR-005).
  - `Resolver::cached_count(&self) -> usize` — introspección para tests.
- Caché guarda solo el `Payload` parseado (`wasmtime::Module` o `wasmtime::component::Component`), **no** Stores ni Instancias. Cada `invoke` sigue creando un `Store` fresco (ADR-005 baseline operativa).
- 4+ tests: cache hit, cache miss (bytes distintos), reset, policies distintas sobre mismo bytes.
- Nuevo bench `cached_cold_start_identity` en `benches/baseline.rs`: cada iteración llama `resolver.load(bytes) + invoke`, pero con el componente ya en caché desde la primera iteración de criterion.
- Actualización de `knowledge/tech/baselines/runtime-baseline.md` con comparación cached vs cold.

**Problem:**
Open Question L147 explícita en el roadmap: la POC recompila dependencias por invocación. Sin resolver, cada ejecución del CLI o executor YAML paga ~247 µs por componente. Con 10 componentes y 100 invocaciones es ~250 ms de puro desperdicio.

**Scope confidence:** medium (API nueva + benchmark + hash content-addressed).

## Acceptance criteria

1. **`Resolver` tipado y público** con los 5 métodos declarados.
2. **Caché por SHA-256:** bytes idénticos devuelven el `Payload` cacheado; bytes distintos pagan parsing una vez y entran a caché.
3. **`reset()` vacía la caché.** `cached_count` lo refleja.
4. **Policies distintas sobre mismo bytes** no colisionan: el `Payload` se reutiliza pero el `Component` resultante lleva la policy pasada en la llamada.
5. **4 tests en `tests/resolver.rs`:**
   - Cache hit: dos `load(bytes)` consecutivos; `cached_count == 1`.
   - Cache miss: `load(bytes_a)` + `load(bytes_b)`; `cached_count == 2`.
   - Reset: `load + load + reset`; `cached_count == 0`.
   - Policy independence: `load_with_policy(bytes, policy_a) + load_with_policy(bytes, policy_b)`; `cached_count == 1`; cada `Component` respeta su propia policy en `invoke`.
6. **Nuevo bench `cached_cold_start_identity`** y mediciones registradas en `runtime-baseline.md`.
7. **Tests existentes (25) siguen verdes.**
8. **Lint y fmt verdes.**
9. **CI remota verde** 3 OS.

## Out of scope

- Caché compartida entre threads / `Arc<RwLock<Resolver>>` (futuro, cuando RM-08 añada concurrency).
- Eviction policy (LRU, size-based) — scope actual: caché unbounded; `reset()` manual.
- Caché persistente (serialización a disco) — futuro.
- Pool de instancias / Stores pre-creados (INIT-08-03 warm pool).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-native-runtime
cargo bench -p agent-native-runtime --bench baseline -- --quick
```

Esperado: 29 tests pass (25 previos + 4 nuevos). Bench `cached_cold_start_identity` significativamente < `cold_start_identity`.

## Definition of Done

- `Resolver` implementado + re-exportado.
- 4 tests nuevos verdes.
- `runtime-baseline.md` con fila nueva de bench cached.
- CI remota verde.
- WI `completed`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-02 |
| ADRs | ADR-001, ADR-005 |
| Prerrequisitos | AN-009, AN-011, AN-012 |
| OQ resuelta | L147 (resolver reutilizable) |

## Learning

**Lo que salió como se esperaba:**

- Content-addressed cache por SHA-256 cumple su promesa: `cached_cold_start_identity` ~2.09 µs vs `cold_start_identity` ~389 µs = **186× speedup**, consistente con el techo teórico predicho por AN-012 (195× entre warm y cold).
- Separación clara entre caché de **Payload** (parsed) y **Component** (con policy) permite que distintas policies compartan la misma entrada de caché sin colisión.
- `reset()` + `cached_count()` dan al llamador control explícito y auditable sobre el ciclo de vida (ADR-005 "fresh instance per invocation" preservado).
- La diferencia entre `cached_cold_start` (2.09 µs) y `warm_invoke` (1.54 µs esta corrida) es solo ~550 ns = overhead de HashMap lookup + SHA-256. Despreciable comparado con el beneficio.

**Lo inesperado:**

- Primera iteración del test `distinct_bytes_grow_the_cache` falló: pensé que cambiar `requirements` en el IR cambiaría los bytes del WASM. **No lo hace** — `requirements` es metadata de trazabilidad que el compilador no emite al binario. Resuelto cambiando el `name` de la función (que sí entra como exported symbol).
- Esto revela una propiedad útil del compiler: cambios en metadata de trazabilidad no invalidan el cache. Vale documentarlo si se vuelve relevante.

**Riesgos materializados:** ninguno. R-RES-01 (fugas de estado) mitigado por diseño: la caché solo guarda Payload parseado, nunca Stores ni Instancias.

**Deuda y seguimientos:**

- Caché unbounded: en producción un `Resolver` long-lived puede crecer sin límite. `reset()` manual es la mitigación actual. INIT-06-03 (ciclo de vida tenant) o un WI dedicado puede añadir LRU/size-based eviction.
- `Resolver` no es `Send + Sync` thread-safe. Comparte entre threads requiere `Arc<Mutex<Resolver>>`; cuando RM-08 añada concurrencia (INIT-08-04 workers), se refina.
- El bench `cached_cold_start_identity` corre con cache pre-primed; valdría añadir también un bench "first miss then hits" para medir el peor caso.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-03-03 (executor YAML)** puede usar `Resolver` para no recompilar entre casos de prueba de la misma suite.
- **INIT-05-01 (CLI)** puede exponer `Resolver` como sesión persistente para `agent-native run` de múltiples componentes.
- **INIT-08-03 (warm pool)** complementa este resolver con instancias pre-creadas; combinados darían el speedup máximo medible.
- **INIT-06-03 (aislamiento multi-tenant)** usa `Resolver::reset()` como la primitiva de limpieza entre tenants.
- **OQ L147** resuelta: la POC recompilaba por invocación; el nuevo producto no.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-013-runtime-resolver.md
crates/runtime/src/resolver.rs
crates/runtime/tests/resolver.rs (4 tests)
crates/runtime/benches/baseline.rs (+1 bench)
knowledge/tech/baselines/runtime-baseline.md (updated)

Next:
INIT-03-03 (executor YAML) — último WI de RM-03
────────────────────────
