---
title: Contrato YAML AN-020 vs YAML runner POC
owner: AN-020
status: baseline-2026-10-18
scope: knowledge/tech
---

# Contrato YAML AN-020 vs YAML runner POC

Este documento es la referencia **normativa** del formato YAML aceptado
por el executor del runtime nuevo (AN-020). Contrasta cada campo con el
formato histórico del POC (`_reference/poc-agent-native/tests/*.yaml`).
Toda incompatibilidad real está declarada aquí, con su destino de
roadmap si procede.

Las secciones siguientes son el esquema completo. Si un campo o
estructura no aparece en ninguna sección, no es parte del contrato
AN-020 y su presencia produce `ExecutorError::InvalidYaml`.

## Campos idénticos

Nombre y semántica preservados exactamente del POC.

| Campo | Tipo | Semántica |
|---|---|---|
| `suite` (raíz) | string | Nombre de la suite para `SuiteReport.suite`. |
| `spec_version` (raíz) | u32 | Versión del esquema de la suite. |
| `cases` (raíz) | lista | Lista de casos. |
| `cases[*].id` | string | Identificador del caso. |
| `cases[*].requirement` | string, opcional | Trazabilidad al requerimiento. |
| `cases[*].function` | string | Nombre del export WIT (`iface#op` o Core WASM). |
| `cases[*].args` | lista de `i32` o `bool` | Argumentos del export. **AN-020**: `bool` añadido (ver sección renombrados). |
| `cases[*].host_state.kv` | mapa `i32: i32` | Estado inicial KV. **AN-020**: `kv` es el único sub-map (ver sección omitidos). |
| `cases[*].expected` (primitivo) | `i32` | Valor esperado del return. |
| `cases[*].expected` (record) | mapa `string: i32` | Record esperado del return (campos por nombre). |
| `cases[*].expected_host_calls` | lista, opcional | Llamadas al host esperadas en orden exacto. |
| `cases[*].expected_host_calls[*].args` | lista `i32` | Args esperados en la llamada. |

Ejemplo:

```yaml
suite: KAN-POC-001-SMOKE
spec_version: 1
cases:
  - id: SMOKE-P01
    requirement: SMOKE-001
    function: identity
    args: [42]
    expected: 42
```

## Campos renombrados o normalizados

Nombre o representación cambiada respecto al POC, con regla documentada.

### `expected_host_calls[*].import` (ex `function`)

**POC:** `expected_host_calls[*].function: check_credit`.

**AN-020:** el campo canónico es `import`. `function` se acepta como
**alias temporal** durante la ventana M3 → M4. Declarar ambos a la vez
en una misma entrada produce `InvalidYaml` con mensaje que menciona
`both` + `import` + `function`.

```yaml
# AN-020 canónico:
- import: host.kv_get
  args: [42]

# Alias POC (equivalente, mientras el alias siga activo):
- function: host.kv_get
  args: [42]

# Rechazado por parse (ambos presentes):
- import: host.kv_get
  function: host.kv_get
  args: [42]
```

### Nombres cortos vs calificados en `import`

**POC:** `import: check_credit` (sólo operation, sin interface prefix).

**AN-020:** el runtime registra la llamada como `interface.operation`
(e.g. `host.kv_get`). Para match contra la expectativa YAML se aplica
esta regla única:

- Si `import` contiene `.` → match literal contra `interface.operation`.
- Si `import` NO contiene `.` → match contra la parte de operación de la
  llamada registrada, ignorando el prefijo de interfaz.
- Si `import` contiene `.` pero una de las dos partes está vacía
  (p. ej. `host.`, `.op`, `.`) → `InvalidYaml` con mensaje
  `ambiguous`.

```yaml
# Ambas formas aceptadas:
- import: host.kv_get     # match literal
  args: [1]
- import: kv_get          # match contra op, ignora prefijo
  args: [1]

# Rechazadas:
- import: 'host.'         # operation vacía
- import: '.op'           # interface vacía
```

### `args` booleans

**POC:** `args: [-1, true]` (ejemplo de `public.yaml`).

**AN-020:** `args` acepta `i32` **y** `bool`. Un `bool` se lifting
automáticamente a `Val::Bool` al invocar el export. Los parámetros
declarados como `bool` en la IR aceptan `true`/`false`; los declarados
como `i32` aceptan enteros. Tipos adicionales (strings, floats, enums)
quedan fuera de AN-020 → INIT-07-02.

```yaml
cases:
  - id: calc-with-flag
    function: calculate_discount
    args: [100, true]
    expected: 10
```

## Campos deliberadamente omitidos

Campos usados por fixtures POC que **no** son parte del contrato AN-020
y producen `InvalidYaml` si aparecen. Cada uno tiene destino de roadmap.

### Sub-maps extra en `host_state`

**POC:** `host_state.credit_limits`, `host_state.tenant_status`,
`host_state.quota`, etc. (ver `public-003.yaml` y
`resource-reservation.yaml`).

**AN-020:** sólo `kv:` es aceptado. Cualquier otra clave en
`host_state` → `InvalidYaml` con mensaje que incluye el nombre de la
clave rechazada. Destino: **INIT-07-02** (host state rico con tipos no
primitivos).

```yaml
# Rechazado:
host_state:
  kv:
    1: 1
  credit_limits:   # <- rechazado
    1: 100
```

### Argumentos no primitivos (strings, floats, listas)

**POC:** no documentado uso en fixtures auditadas, pero el formato
no restringe explícitamente.

**AN-020:** sólo `i32` y `bool`. Destino: **INIT-07-02**.

### Records anidados en `expected`

**POC:** no se observa uso.

**AN-020:** `expected` como record acepta sólo mapa `string: i32`
plano. Un valor no-i32 (sub-map, lista, bool) en un campo esperado
produce fallo de parse. Destino: **RM-07** si surge demanda.

## Estructuras-raíz no soportadas

Formas de raíz que la POC permite y AN-020 **no**.

### Raíz sin `suite` + `spec_version`

**POC:** `narrative-005.yaml` empieza directo con `cases:` (sin `suite`
ni `spec_version`).

**AN-020:** ambos campos son **obligatorios**. Omitir cualquiera
produce `InvalidYaml`. Decisión: fail-explicit sobre el schema; las
fixtures POC narrativas se adaptan con una línea `suite:` + una línea
`spec_version:` o se clasifican como `aplazado` en la matriz
`poc-equivalence-matrix.md`.

```yaml
# Rechazado (sin suite):
cases:
  - id: foo
    ...

# Rechazado (sin spec_version):
suite: my-suite
cases:
  - id: foo
    ...
```

### `acceptance.public` como sección-raíz

**Clarificación documental:** `acceptance.public` **no** es un formato
de suite del runner POC. El término `acceptance` aparece sólo en el
*orchestrator* del POC (`crates/orchestrator/src/interpreter.rs` y
`src/lib.rs`) como campo `acceptance_criteria` de la extracción desde
markdown. No es una clave YAML del formato de suites del runner, ni
del AN-020.

AN-020 **no adopta** `acceptance.public` como clave YAML. Si una
futura fuente de verdad lo requiere, se tratará como un formato
distinto (no una suite de executor) con WI dedicado.

### Campos desconocidos en raíz, `cases[*]` o `host_state`

**AN-020:** `Suite`, `Case` y `HostState` llevan
`#[serde(deny_unknown_fields)]`. Campos extra producen `InvalidYaml`.
Esto incluye futuras extensiones del POC no previstas aquí.

Esta decisión es deliberada: el runtime es la fuente de verdad del
contrato; cualquier extensión del YAML entra por un WI que amplíe este
documento primero.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| WI | AN-020 — Cierre técnico M3 |
| Hallazgo origen | H4 (incompatibilidad YAML real con POC) |
| Decisiones humanas incorporadas | OQ-01 (import canónico + alias), OQ-02 (sub-maps extra rechazados) |
| ADRs | ADR-004 (fail-explicit) |
| Destinos de roadmap para omitidos | INIT-07-02 (host state rico), RM-07 (records anidados), M4 (esclusas) |
