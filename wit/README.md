# WIT contracts

Esta carpeta declara la **convención de ubicación** para archivos WIT en el workspace. La convención está en vigor aunque algunas categorías estén vacías hoy — ver §4 "Estado actual".

## 1. Las tres categorías

### `wit/public/` — Contratos estables del producto

Interfaces WIT que el producto expone como contrato público: lo que un consumidor externo puede escribir contra Agent-Native. Cambios aquí son **semánticamente visibles**: añadir, renombrar o quitar funciones rompe compatibilidad.

Reglas:

- Nombres de package reservados a los namespaces oficiales del producto (p. ej. `agent-native:*`).
- Cada archivo en esta carpeta tendrá un ADR asociado cuando se materialice.
- Un lint CI dedicado validará esta carpeta cuando reciba su primer archivo (ver §4).

**Qué NO va aquí:** fixtures de prueba (usan dominios inventados como `an:test/*`, `kaddo:*`, `an:probe/*`), ni recursos embebidos de crates (`include_str!`).

### `wit/internal/` — Recursos embebidos de crates

WIT files que un crate incrusta con `include_str!` como recurso de compilación. **No son contratos del producto** — son detalles de implementación. Hoy residen bajo cada crate (p. ej. `crates/compiler/wit/order.wit`, fallback IR v0.4).

Esta carpeta queda **reservada** para el caso futuro en que se consolide un recurso WIT compartido entre crates. Mientras tanto, los recursos embebidos permanecen con sus crates respectivos y no se mueven aquí.

### `wit/fixtures/` — Fixtures de prueba

Archivos WIT que exercitan el pipeline (verifier, compiler, runtime) con dominios inventados: `kaddo:canal`, `an:test/*`, `an:probe/*`, `an:nested/*`, etc. **No son contratos del producto** y nunca aparecen en `wit/public/`.

Hoy residen con los tests que los consumen bajo `crates/compiler/tests/fixtures/wit/` (co-ubicados con sus fixtures IR). Esta carpeta queda **reservada** para el caso en que un fixture sea reutilizado por múltiples crates (hoy el caso no existe).

## 2. Reglas para añadir un `.wit` nuevo

1. ¿Es una interfaz que un consumidor externo escribirá contra el producto? → `wit/public/`. Documenta con ADR + lint CI.
2. ¿Es un recurso embebido por un crate concreto (`include_str!`)? → junto al crate (`crates/<name>/wit/`). **No** aquí.
3. ¿Es una fixture de prueba? → junto al test que la consume (`crates/<name>/tests/fixtures/wit/`). **No** aquí.
4. ¿Es un fixture compartido entre múltiples crates de test? → `wit/fixtures/`. Caso raro.

## 3. Qué pasa con los fixtures actuales

Permanecen donde están porque funcionan y están correctamente aislados:

| Archivo | Ubicación actual | Categoría conceptual |
|---|---|---|
| `crates/compiler/wit/order.wit` | compiler crate (embedded via `include_str!`) | `wit/internal/` conceptual |
| `crates/compiler/tests/fixtures/wit/program-005.wit` | compiler tests | `wit/fixtures/` conceptual |
| `crates/compiler/tests/fixtures/wit/probe-use-types.wit` | compiler tests | `wit/fixtures/` conceptual |
| `crates/compiler/tests/fixtures/wit/nested-records.wit` | compiler tests | `wit/fixtures/` conceptual |

Los tests de `agent-native-runtime` resuelven estos fixtures vía `CompileContext` apuntando a `crates/compiler/` (AN-016). Mover archivos a `wit/fixtures/` requeriría cambiar el `CompileContext` a workspace root y actualizar los paths del IR; el beneficio no justifica el churn hasta que haya al menos un fixture compartido entre crates distintos.

## 4. Estado actual y lint CI

- `wit/public/` — **vacío.** No existen contratos públicos del producto. Los introducirá RM-05 (INIT-05-01: subcomandos CLI con contrato estable) o RM-07 (INIT-07-01: WASI stdio como capability expuesta).
- `wit/internal/` — **vacío.** El único WIT embebido vive donde debe (`crates/compiler/wit/order.wit`).
- `wit/fixtures/` — **vacío.** Los fixtures de test actuales viven junto a sus crates.

**Lint CI diferido:** cuando `wit/public/` reciba su primer archivo, se añadirá un paso en `.github/workflows/ci.yml` que valide:

1. Ningún archivo en `wit/public/` usa namespaces de fixture (`kaddo:canal`, `an:test/*`, `an:probe/*`, `an:nested/*`).
2. Todo `.wit` en `wit/public/` tiene su ADR asociado en `knowledge/tech/decisions/`.
3. Los packages declarados en `wit/public/` siguen el namespace oficial del producto.

Hasta entonces, la convención se mantiene por acuerdo documental.

## 5. Referencias

- [ADR-003 — Semantic IR v0.5 baseline](../knowledge/tech/decisions/ADR-003-semantic-ir-v05-baseline.md)
- [Roadmap INIT-02-03](../knowledge/delivery/roadmap.md) — origen de esta convención.
- [AN-014 Learning](../knowledge/delivery/work-items/completed/AN-014-e2e-semantic-ir-v05-component-model.md) — por qué los fixtures con `use types.{...}` ejercitan el pipeline Component Model completo.
- [AN-016 Learning](../knowledge/delivery/work-items/completed/AN-016-compiler-robustness-memory-model.md) — `CompileContext` resuelve `wit_file` del IR independiente del cwd.
