---
type: baseline
subtype: runtime
project_state: ai-assisted
generated_by: implementation-agent
template_version: 2
work_item: AN-017 (consolidación de AN-012 + AN-013)
---

# Runtime baseline

> **Referencia histórica (AN-017).** El snapshot Wasmtime 27 / `--quick` de abajo
> se conserva íntegro como historial. Para nuevas decisiones se sustituye por la
> [publicación medida candidata de AN-055](../../../docs/benchmarks/README.md),
> basada en Wasmtime 29.0.1 y corridas completas cross-OS. Esa publicación sigue
> pendiente de certificación humana; las filas inestables no constituyen una
> línea base certificada. No interpretar el cambio de referencia como mejora
> cuantificada entre versiones, máquinas o métodos distintos.

Línea base técnica de `agent-native-runtime`. **No es un SLA.** Es la medición mínima requerida por el principio del roadmap #5 ("mediciones antes que optimización") y la contraparte local de la futura matrix cross-OS que materialice INIT-08-01/02.

Cualquier optimización futura debe **documentar su mejora cuantificada contra la fila específica** de esta tabla; afirmaciones agregadas (p. ej. "mejora del runtime") sin especificar etapa y tipo de artefacto no son aceptables.

## Alcance y disclaimers

- **No cross-OS.** Esta corrida es Windows 11 local sobre una workstation personal. INIT-08-01/02 (RM-08) formaliza la matrix Linux/Windows/macOS en hardware controlado.
- **No production hardware.** Los números no se trasladan a servidores dedicados sin re-medición.
- **No comparación con implementaciones nativas.** Comparar con Rust/Go/TypeScript pertenece a INIT-08-02 con metodología justa.
- **No instrumentación de memoria.** Wasmtime 27 no expone bytes allocados sin `ResourceLimiter`; diferido a INIT-06-02 (ver `knowledge/tech/discovery/memory-model-notes.md`).
- **No concurrencia.** Soak tests y workers pertenecen a INIT-08-04.
- **Mediciones con `criterion --quick`.** La corrida larga (`cargo bench ... --bench baseline`, sin `--quick`) produce varianzas más estrechas; el orden de magnitud se mantiene.

## Metodología

- **Herramienta:** [`criterion`](https://bheisler.github.io/criterion.rs/book/) 0.5 como `[dev-dependencies]` del crate `agent-native-runtime`.
- **Fixture Core WASM — `identity`:** función `(i32) -> i32` compilada desde IR v0.1 vía el shim `compile_ir_to_wasm` (el IR no referencia `wit_file` externo, así que el cwd no afecta el output). Diseñada para que el costo de dominio sea despreciable; mide el pipeline del runtime.
- **Fixture Component Model — `point_sum`:** función `(i32, i32) -> record { sum, doubled }` compilada desde IR v0.5 (`program-005`) con contrato WIT (interface `an:test/ops@0.1.0`). Mide el camino completo con interfaces WIT y retorno de record. Al referenciar un `wit_file` externo, se compila vía `compile_ir_to_wasm_with_context` + `CompileContext` apuntando a `crates/compiler/` (AN-016); el fixture `identity` **no** usa esta API.
- **Separación por etapa:** cada bench aísla una porción del pipeline, de modo que optimizaciones futuras pueden compararse contra la fila que afectan.
- **Store per invocación:** todos los benches de `*_run_*` crean un `wasmtime::Store` fresh por invocación (ADR-005 baseline operativa). El `warm_run` cachea el `wasmtime::Component`/`Module` parseado pero no la instancia.

### Nomenclatura

```text
<stage>_<path>_<fixture>
```

| Prefijo | Significado |
|---|---|
| `engine_construct` | Solo `Engine::new()`. |
| `load_<path>_<fixture>` | `Engine::new()` + `load_component(bytes)` sin invocar. |
| `cold_run_<path>_<fixture>` | `Engine::new()` + `load_component` + `invoke` — pago completo del cold path. |
| `warm_run_<path>_<fixture>` | Engine y artefacto pre-construidos; mide solo `invoke` (nuevo Store cada vez). |
| `cache_hit_run_<path>_<fixture>` | Engine pre-construido + `Resolver` con cache primed; mide `Resolver::load` (hit) + `invoke`. |

donde `<path>` ∈ {`core`, `component_model`} y `<fixture>` ∈ {`identity`, `point_sum`}.

## Hardware y toolchain

| Campo | Valor |
|---|---|
| CPU | Workstation local (anonimizada; detalles cuando se centralice inventario de hardware en INIT-08-01) |
| OS | Windows 11 Pro for Workstations 10.0.26300 |
| Rust toolchain | `rustc 1.85.0 (4d91de4e4 2025-02-17)` — pinneado por `rust-toolchain.toml` |
| Wasmtime | 27.0 (features: cranelift, runtime, component-model) |
| Modo build | `--release` (criterion default) |
| Método | criterion `--quick` (primera medición rápida) |

## Resultados — snapshot AN-017 (2026-10-18)

### Tabla principal

| Bench | Etapa | Path | Fixture | Mediana | Rango |
|---|---|---|---|---|---|
| `engine_construct` | Engine | — | — | **~2.08 µs** | 2.05 – 2.09 µs |
| `load_core_identity` | Load | Core WASM | identity | **~229 µs** | 228 – 234 µs |
| `load_component_model_point_sum` | Load | Component Model | point_sum | **~500 µs** | 499 – 501 µs |
| `cold_run_core_identity` | Cold run | Core WASM | identity | **~230 µs** | 230 – 230 µs |
| `cold_run_component_model_point_sum` | Cold run | Component Model | point_sum | **~601 µs** | 601 – 610 µs |
| `warm_run_core_identity` | Warm run | Core WASM | identity | **~1.65 µs** | 1.64 – 1.71 µs |
| `warm_run_component_model_point_sum` | Warm run | Component Model | point_sum | **~46.7 µs** | 46.4 – 47.9 µs |
| `cache_hit_run_core_identity` | Cache-hit run | Core WASM | identity | **~1.89 µs** | 1.89 – 1.92 µs |
| `cache_hit_run_component_model_point_sum` | Cache-hit run | Component Model | point_sum | **~45.8 µs** | 45.4 – 45.9 µs |

### Observaciones acotadas al alcance medido

Las medianas publicadas vienen de `criterion --quick`: son snapshots rápidos de un solo pase sobre workstation personal. Las comparaciones a continuación son **aproximadas** (orden de magnitud + sign) y no deben leerse como mediciones causales con precisión ns:

- **Cold run ≫ warm run (Core WASM identity):** el cold path está mayoritariamente dominado por `load`. La reducción disponible para invocaciones repetidas del mismo artefacto es la diferencia entre cold y warm; AN-013 (`Resolver`) la materializa, lo que se ve en `cache_hit_run_core_identity` ≈ `warm_run_core_identity` en orden de magnitud.
- **`cache_hit_run_*` del mismo orden que `warm_run_*`:** el overhead del cache lookup (`HashMap<[u8;32], _>` + SHA-256 del input) no cambia el orden de magnitud de una invocación. La diferencia puntual en µs varía entre corridas `--quick` y no se afirma como "overhead medido".
- **Component Model `point_sum` es significativamente más lento que Core WASM `identity`:** se observa en todas las etapas comparables (load, cold-run, warm-run, cache-hit-run). Esto NO es "Component Model es lento"; es un conjunto de factores entremezclados: (a) `point_sum` retorna un record, no un primitivo, (b) la invocación cruza el límite de interfaz WIT con conversión del Canonical ABI, (c) el artefacto compilado es más grande. **Comparar directamente los dos fixtures excede el alcance de esta tabla.** INIT-08-02 formalizará comparaciones controladas.
- **La instanciación no se mide como etapa separada.** La fase que ocurre entre `load_component` y la primera `invoke` (crear `Store`, instanciar en el linker vacío, resolver imports) está sumada en el delta `cold_run − load`. Un bench de "instantiate aislado" no existe en esta suite; INIT-08-01 puede añadirlo si justifica el valor.

### Qué NO mide esta tabla

- **Memoria por instancia** — diferida a INIT-06-02 cuando `ResourceLimiter` esté integrado.
- **Comparación con nativo Rust/Go/TS** — INIT-08-02 con metodología justa.
- **Cross-OS** — INIT-08-01 formaliza la matrix.
- **Concurrencia / soak tests** — INIT-08-04.
- **Allocación masiva en una invocación** — IR v0.5 no permite construir fixtures de estrés (sin loops/listas); diferido a IR v0.6+ (RM-07).
- **Records anidados** — IR v0.5 rechaza `record` dentro de `record` por diseño; ver AN-016 Learning y `memory-model-notes.md` §2.
- **Compromisos de producción** — ninguno. Estos números son snapshot local, no SLA.

## Cómo reproducir

```bash
# Compilar los benchmarks (incluido en CI --all-targets)
cargo bench -p agent-native-runtime --no-run

# Snapshot rápido local (9 benches, ~2 min total en workstation)
cargo bench -p agent-native-runtime --bench baseline -- --quick

# Corrida completa criterion (varianzas estrechas, ~15 min)
cargo bench -p agent-native-runtime --bench baseline
```

Los números publicados arriba son `--quick`. Al añadir una fila al Historial, indica el método usado y el hardware exacto.

## Qué desbloquea este documento

- **INIT-03-02 (ya cerrado)** demostró 186× con `Resolver`. El bench `cache_hit_run_*` aquí captura la misma invariante en términos del nombre nuevo.
- **INIT-08-01 (RM-08)**: reutiliza este `baseline.rs` como base; añade matrix cross-OS y publicación periódica.
- **INIT-08-03 (RM-08)**: cuantificará el beneficio del warm pool con esta tabla como referencia.
- **INIT-05-01 (RM-05)**: el `--format=json` del CLI debe exponer estos escenarios bajo un flag `--benchmark` o similar; fuera de alcance de AN-017.

## Historial

| Fecha | Commit / WI | Nota |
|---|---|---|
| 2026-10-17 | AN-012 | Línea base inicial local, snapshot `--quick`: `cold_start_identity` ~389 µs, `warm_invoke_identity` ~1.99 µs, `load_only` ~247 µs. |
| 2026-10-17 | AN-013 | Añadido `cached_cold_start_identity` ~2.09 µs. 186× speedup sobre `cold_start_identity`. |
| 2026-10-18 | AN-017 | **4 benches renombrados** (`cold_start_identity` → `cold_run_core_identity`, `warm_invoke_identity` → `warm_run_core_identity`, `cached_cold_start_identity` → `cache_hit_run_core_identity`, `load_only` → `load_core_identity`) + **5 benches añadidos** (`engine_construct`, `load_component_model_point_sum`, `cold_run_component_model_point_sum`, `warm_run_component_model_point_sum`, `cache_hit_run_component_model_point_sum`). Tabla segregada por path y etapa. **Nota de reproducibilidad:** las medianas AN-017 difieren de AN-012/013 en magnitudes variables (p. ej. `cold_run_core_identity` ~230 µs vs `cold_start_identity` ~389 µs, delta observado ≈ −41 %, mayor que el ruido típico de `--quick`). **No se afirma como cambio estructural demostrado ni se atribuye a una causa concreta**: se requiere una comparación controlada posterior (INIT-08-01, RM-08) con hardware y método no-`--quick` para discernir ruido de workstation, cambios de dependencias y evolución del compilador. Los números AN-012/013 permanecen como **históricos locales**, no como regresión demostrada. |

### Mapa de nombres antiguos → nuevos (preservado tras AN-017)

| Antes de AN-017 | Después de AN-017 |
|---|---|
| `cold_start_identity` | `cold_run_core_identity` |
| `warm_invoke_identity` | `warm_run_core_identity` |
| `cached_cold_start_identity` | `cache_hit_run_core_identity` |
| `load_only` | `load_core_identity` |
| — (no existía) | `engine_construct` |
| — (no existía) | `load_component_model_point_sum` |
| — (no existía) | `cold_run_component_model_point_sum` |
| — (no existía) | `warm_run_component_model_point_sum` |
| — (no existía) | `cache_hit_run_component_model_point_sum` |

Los nombres antiguos están **eliminados del código** (`baseline.rs` ya no los expone). Esta tabla queda como referencia histórica para interpretar los commits anteriores a AN-017.
