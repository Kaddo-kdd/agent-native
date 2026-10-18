---
type: baseline
subtype: runtime
project_state: ai-assisted
generated_by: implementation-agent
template_version: 1
work_item: AN-012
---

# Runtime baseline (INIT-03-06)

Línea base técnica de `agent-native-runtime`. **No es un SLA.** Es la medición mínima requerida por el principio del roadmap #5 ("mediciones antes que optimización") antes de abrir INIT-03-02 (resolver reutilizable), INIT-06-03 (reutilización de instancias) y RM-08 (harness formal cross-OS).

Cualquier optimización futura debe **documentar su mejora cuantificada contra esta línea base** para considerarse valor neto.

## Alcance y disclaimers

- **No cross-OS.** Esta corrida es Windows 11 local. RM-08 (INIT-08-01/02) formaliza matrix Linux/Windows/macOS en hardware comparable.
- **No production hardware.** Workstation local; los números no se trasladan a servidores dedicados sin re-medición.
- **No comparación con nativo.** Comparar con implementaciones Rust/Go/TS nativas pertenece a INIT-08-02 con metodología justa.
- **Varianza:** mediciones de criterion `--quick` (~1 segundo por bench). Resultados completos (`cargo bench` sin `--quick`) producen varianzas más estrechas pero el orden de magnitud se mantiene.
- **Memoria por instancia:** diferida a INIT-08-02 (requiere instrumentación con `wasmtime::ResourceLimiter` o profilers).

## Metodología

- **Fixture:** función `identity(i32) -> i32` compilada desde IR v0.1 (Core WASM). Diseñada para que el costo de dominio sea despreciable y la medición refleje el pipeline del runtime (`Engine::new`, parsing/validación de WASM, instanciación en `Store`, lookup+call de la función exportada).
- **Herramienta:** [`criterion`](https://bheisler.github.io/criterion.rs/book/) 0.5 como `[dev-dependencies]` + `[[bench]] name = "baseline" harness = false`.
- **Benchmarks:**
  - `cold_start_identity`: cada iteración construye `Engine`, carga el componente e invoca una vez.
  - `warm_invoke_identity`: el componente queda pre-cargado antes del bench; cada iteración solo invoca.
  - `load_only`: `Engine::new` + `load_component` sin invocar — aisla el costo de parsing/validación.
- **Modo:** `cargo bench -p agent-native-runtime --bench baseline -- --quick` (first local snapshot).

## Hardware y toolchain

| Campo | Valor |
|---|---|
| CPU | Workstation local (anonimizado — ver `knowledge/tech/baselines/README.md` cuando se centralice) |
| OS | Windows 11 Pro for Workstations 10.0.26300 |
| Rust toolchain | `rustc 1.85.0 (4d91de4e4 2025-02-17)` — pinneado por `rust-toolchain.toml` |
| Wasmtime | 27.0 (features: cranelift, runtime, component-model) |
| Modo build | `--release` (criterion default) |

## Resultados (snapshot inicial)

| Operación | Mediana | Rango (criterion `--quick`) |
|---|---|---|
| `cold_start_identity` (Engine + load + invoke) | **~389 µs** | 382.7 – 390.5 µs |
| `warm_invoke_identity` (invoke sobre componente pre-cargado) | **~1.99 µs** | 1.98 – 1.99 µs |
| `load_only` (Engine + load, sin invocar) | **~247 µs** | 245.7 – 247.6 µs |
| `cached_cold_start_identity` (Resolver con cache hit + invoke; AN-013) | **~2.09 µs** | 2.05 – 2.10 µs |

### Observaciones

- El **cold start** está dominado (~63%) por `load_only` (parsing + validación WASM + construcción de módulo Cranelift). El resto es la creación del `Engine` y la invocación en sí.
- La **invocación warm es ~195× más rápida que la cold** — era el techo teórico cuando se registró el snapshot inicial.
- **AN-013 (Resolver content-addressed)** materializa ese techo: `cached_cold_start_identity` ~2.09 µs **vs** `cold_start_identity` ~389 µs = **~186× más rápido** sobre el cold uncached. Es ~5% más lento que `warm_invoke` puro (overhead del lookup en HashMap + hash SHA-256), diferencia despreciable al escalar a varios componentes.
- **Memoria** no instrumentada aún; se añade cuando INIT-06-02 (fuel + instance limits) exponga la API o cuando INIT-08-02 lo requiera.

## Cómo reproducir

```bash
# Compilar los benchmarks (incluido en CI --all-targets)
cargo bench -p agent-native-runtime --no-run

# Snapshot rápido local (3 operaciones, ~5s total)
cargo bench -p agent-native-runtime --bench baseline -- --quick

# Corrida completa criterion (varianza estrecha, ~90s)
cargo bench -p agent-native-runtime --bench baseline
```

## Qué desbloquea este documento

- **INIT-03-02** (resolver reutilizable): puede comprometerse a "mejorar warm invoke en X%" o "reducir cold start en Y µs" con esta línea base como referencia.
- **INIT-08-01/02** (harness formal cross-OS): parte de esta metodología; añade matrix de OS, hardware controlado y publicación periódica.
- **INIT-08-03** (caché de componentes): debe cuantificar su mejora contra el `cold_start_identity` aquí registrado.

## Historial

| Fecha | Commit | Nota |
|---|---|---|
| 2026-10-17 | (AN-012) | Línea base inicial. Windows local. Snapshot `--quick`. |
| 2026-10-17 | (AN-013) | Añadido `cached_cold_start_identity` ~2.09 µs. 186× speedup sobre `cold_start_identity`. |

Snapshots futuros deben añadirse como filas, no reemplazar las existentes.
