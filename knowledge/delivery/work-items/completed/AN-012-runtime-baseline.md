---
id: AN-012
type: chore
status: completed
completed_at: 2026-10-17
title: Línea base técnica del runtime (cold / warm / memoria)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
code:
  - crates/runtime/Cargo.toml
  - crates/runtime/benches/baseline.rs
  - knowledge/tech/baselines/runtime-baseline.md
source_roadmap_initiative: INIT-03-06
source_work_item_candidate:
  - WI-CANDIDATE-03-06-01
  - WI-CANDIDATE-03-06-02
  - WI-CANDIDATE-03-06-03
related_domain: Runtime & Memory
domains:
  - Runtime & Memory
  - Performance
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Métricas numéricas de cold start, warm invocation y memoria baseline del
  runtime, obtenidas antes de introducir caché/warm pool. Entrada obligatoria de
  INIT-03-02 (resolver reutilizable) y RM-08 (harness formal cross-OS).
risks:
  - >-
    R-BL-01: criterion crate puede arrastrar dep conflicts; mitigado con cargo
    update si necesario
  - >-
    R-BL-02: números medidos aquí NO son SLAs — son referencia local, no
    cross-OS. RM-08 formaliza cross-OS.
dependencies:
  - AN-009
  - AN-011
source_signals:
  - 'Roadmap INIT-03-06: línea base técnica antes de optimizar'
  - 'Principio del roadmap #5: mediciones antes que optimización'
  - >-
    ADR-003: baseline Wasmtime 27 [open] — este bench también documenta su
    comportamiento
decision_candidates: []
related_decisions:
  - ADR-003
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-012 — Línea base técnica del runtime

**Actor y resultado:**
Un contribuidor ejecuta `cargo bench -p agent-native-runtime` y obtiene números reproducibles de 3 operaciones del runtime: cold start (Engine + load + first invoke), warm invocation (invokes subsecuentes sobre un componente ya cargado) y memoria baseline estimada por invocación. Los números se publican en `knowledge/tech/baselines/runtime-baseline.md` como línea base del hardware local (no SLA).

**Current behavior:**
No existen benchmarks. Cualquier decisión sobre caché (INIT-03-02, INIT-08-03) o reuso de instancias (INIT-06-03) sería a ciegas.

**Target behavior:**
- `crates/runtime/benches/baseline.rs` con 3 benchmarks `criterion`:
  - `cold_start_identity`: `Engine::new()` + `load_component(identity_wasm)` + `invoke("identity", [S32(42)])`.
  - `warm_invoke_identity`: componente pre-cargado; cada iteración solo invoca.
  - `load_only`: solo `Engine::new()` + `load_component`, sin invocar — aisla el costo de parsing/validación de WASM.
- `crates/runtime/Cargo.toml`: añade `criterion = "0.5"` como `[dev-dependencies]` + sección `[[bench]]`.
- `knowledge/tech/baselines/runtime-baseline.md`: documento con metodología, hardware, resultados numéricos, cómo reproducir y disclaimers.
- No se exige que estos benchmarks corran en CI (cost + varianza en runners compartidos). Son snapshot local; INIT-08-01 formaliza harness cross-OS.

**Problem:**
Sin línea base, INIT-03-02 (resolver cacheado) no puede demostrar su beneficio cuantificado, y RM-08 arrancaría sin referencia histórica.

**Scope confidence:** medium (criterion dep nueva; baseline numérica depende de hardware local).

## Acceptance criteria

1. **3 benchmarks implementados** en `benches/baseline.rs` usando `criterion`.
2. **`cargo bench -p agent-native-runtime --no-run`** compila sin errores (ejecución completa es best-effort local).
3. **Al menos una corrida local** registrada en `knowledge/tech/baselines/runtime-baseline.md` con: hardware (CPU/RAM/OS), toolchain Rust/Wasmtime, metodología, 3 valores (cold / warm / load-only) con unidades (µs o ms), varianza observada.
4. **Documento incluye disclaimers**: no es SLA, no es cross-OS, no es production hardware.
5. **Documento referenciado** desde `knowledge/delivery/roadmap.md` (sección RM-08) y desde `knowledge/tech/current-state.md`.
6. **Tests existentes (25) siguen verdes.**
7. **Lint y fmt verdes.**
8. **CI remota verde** (benches compilan en `--all-targets`; no se ejecutan en CI).

## Out of scope

- Memoria **medida** por instancia (requiere `wasmtime::ResourceLimiter` o profilers específicos; INIT-06-02 o INIT-08-02 lo cubren). Aquí **estimamos** memoria via `std::alloc::System` o la dejamos como "pendiente de instrumentación".
- Benchmarks cross-OS (INIT-08-01).
- Comparación con implementaciones nativas Rust/Go/TS (INIT-08-02).
- Soak tests, concurrencia (INIT-08-04).
- CI gate de performance (RM-08).

## Validation

```bash
cargo bench -p agent-native-runtime --no-run
cargo bench -p agent-native-runtime -- --quick    # opcional, local only
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Esperado: compila, no rompe tests existentes.

## Definition of Done

- `benches/baseline.rs` creado.
- Doc baseline publicado con una corrida local real.
- Referencia desde roadmap/current-state.
- CI remota verde.
- WI `completed`.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-03 |
| Roadmap iniciativa | INIT-03-06 |
| ADRs | ADR-003, ADR-005 |
| Habilita | INIT-03-02 (resolver), INIT-08-* (harness formal) |

## Learning

**Lo que salió como se esperaba:**

- `criterion` 0.5 integra sin dep conflicts tras el pin de `encoding_rs` que AN-009 ya había añadido al `Cargo.lock`.
- Los 3 benchmarks (cold / warm / load-only) producen números limpios con varianza < 2% en modo `--quick`.
- La separación `load_only` vs `cold_start` fue iluminadora: ~63% del cold start es puro load (parsing + validation + cranelift compile).
- `warm_invoke` ~195× más rápido que cold confirma que el objetivo de INIT-03-02/08-03 (resolver + warm pool) tiene techo real cuantificable.

**Lo inesperado:**

- Cargo pasa args de bench al **primer binary de benches**, no al que uno quiere; para pasar `--quick` al bench propio hay que usar `--bench baseline --` explícitamente. Documentado en el runtime-baseline.md.
- `cargo bench --no-run` tardó ~3 min en primera compilación por cranelift completo en modo release. En CI de PRs esto podría ser caro — por eso NO se añade el bench como gate CI (consistente con el "Out of scope").
- La memoria por instancia resultó no instrumentable sin más superficie: Wasmtime no expone un hook directo de "bytes allocated" accesible desde fuera de un ResourceLimiter. Diferida con nota explícita a INIT-06-02/08-02.

**Riesgos materializados:** ninguno. R-BL-01 nulo (criterion compila), R-BL-02 preservado como disclaimer prominente en el doc.

**Deuda y seguimientos:**

- Memoria por instancia pendiente: lo tomará INIT-06-02 (fuel + instance limits) o INIT-08-02 (full baseline cross-OS).
- Hardware anonimizado en el doc; cuando se centralice el inventario de hardware (RM-08) se añadirá sección específica.
- El bench corre con el toolchain pinneado 1.85 y Wasmtime 27; cualquier bump requiere nueva corrida y entrada en el historial.

**Lo que esta iniciativa establece para las siguientes:**

- **INIT-03-02** puede abrir con el objetivo cuantificado: "reducir cold start por invocación amortizando el componente parse/compile a través de N invocaciones".
- **INIT-08-01** reutiliza este `benches/baseline.rs` como base, extendiéndolo con matrix cross-OS y publicación periódica.
- **INIT-08-03** (caché / warm pool) tiene un objetivo medible: acercar cold start a warm invoke.

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-012-runtime-baseline.md
crates/runtime/benches/baseline.rs
knowledge/tech/baselines/runtime-baseline.md

Next:
INIT-03-03 (executor YAML) o INIT-03-02 (resolver)
────────────────────────
