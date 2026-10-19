---
title: Contrato YAML AN-020 vs YAML runner POC
owner: AN-020
status: baseline-2026-10-19 (patch — corrige dirección `import`↔`function`, documenta 3 raíces POC)
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

Nombre y semántica preservados del POC (donde se usan en fixtures POC).

| Campo | Tipo | Semántica | Nota |
|---|---|---|---|
| `cases` (raíz, mapa) | lista | Lista de casos. | POC también admite otras dos formas de raíz (ver sección "Estructuras-raíz no soportadas"). |
| `cases[*].id` | string | Identificador del caso. | — |
| `cases[*].requirement` | string, opcional | Trazabilidad al requerimiento. | — |
| `cases[*].function` | string | Nombre del export (POC: Core o Component Model; AN-020: `iface#op` para CM, nombre plano para Core). | — |
| `cases[*].args` | lista de `i32` y/o `bool` | Argumentos del export. POC usa booleans en `public.yaml` (`args: [-1, true]`). | AN-020 restaura `bool` tras la regresión AN-019 que sólo aceptaba `i32`. |
| `cases[*].host_state.kv` | mapa `i32: i32` | Estado inicial KV. | AN-020: **sólo** `kv:`; otros sub-maps POC rechazados (ver "Omitidos"). |
| `cases[*].expected` (primitivo) | `i32` | Valor esperado del return. | — |
| `cases[*].expected` (record) | mapa `string: i32` | Record esperado del return (campos por nombre). | AN-020 compara por nombre + tipo + valor; detecta faltantes y adicionales. |
| `cases[*].expected_host_calls[*].import` | string | Nombre del import del host llamado (`interface.operation` o `operation` corta). | **Campo canónico del POC**, confirmado en `runner/src/main.rs:53` y en fixtures. AN-019 regresó accidentalmente a `function`; AN-020 restaura. |
| `cases[*].expected_host_calls[*].args` | lista `i32` | Args esperados en la llamada. | — |

### Campos **exigidos sólo por AN-020** (no obligatorios en POC)

| Campo | Tipo | Comentario |
|---|---|---|
| `suite` (raíz) | string | **No existe en `parse_cases` del POC** (L341-L352). AN-020 lo hace obligatorio para alimentar `SuiteReport.suite`. |
| `spec_version` (raíz) | u32 | Idem. AN-020 lo hace obligatorio para versionar el schema. |
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

### `expected_host_calls[*].import` (campo canónico; `function` fue regresión AN-019)

**POC original (`runner/src/main.rs:53`):** `expected_host_calls[*].import: check_credit`.
Ver fixture `public-003.yaml` y los campos del struct `TestCaseInput`
en el runner POC: `function: String` identifica el **export** del
componente, `import: String` dentro de `ExpectedHostCall` identifica el
**host import**. Los dos campos coexisten con roles distintos.

**AN-019 (regresión 2026-10-18):** nombró el campo de la llamada al
host como `function` en vez de `import`, duplicando el nombre que ya
se usaba al nivel de case para el export. Esta regresión rompía la
compatibilidad semántica con el formato POC.

**AN-020 (restaura canónico + alias):** vuelve al nombre POC `import`
para `expected_host_calls[*]`. `function` se mantiene como **alias
temporal** sólo para no romper las suites internas migradas desde
AN-019 durante la ventana M3 → M4. Declarar ambos a la vez en una
misma entrada produce `InvalidYaml` con mensaje que menciona `both` +
`import` + `function`.

```yaml
# POC original y canónico AN-020:
- import: host.kv_get
  args: [42]

# Alias temporal (fixtures AN-019 pre-corrección):
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

### `args` booleans (restauración tras regresión AN-019)

**POC:** `args: [-1, true]` (ejemplo de `public.yaml`). El runner POC
siempre aceptó booleans de forma nativa.

**AN-019 (regresión):** sólo aceptaba `Vec<i32>`, rompiendo las
fixtures POC con booleans.

**AN-020:** restaura la aceptación nativa de `bool`. `args` acepta
`i32` **y** `bool`. Un `bool` se lifting automáticamente a `Val::Bool`
al invocar el export. Los parámetros declarados como `bool` en la IR
aceptan `true`/`false`; los declarados como `i32` aceptan enteros.
Tipos adicionales (strings, floats, enums) quedan fuera de AN-020 →
INIT-07-02.

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

### Tres formas de raíz admitidas por `parse_cases` del POC

**POC (`runner/src/main.rs::parse_cases` L341-L352):** el runner acepta
**tres** estructuras-raíz distintas:

1. Mapa con clave `cases:` — la más común (p. ej. `smoke.yaml`,
   `public.yaml`).
2. Mapa con clave `acceptance:` que contiene `public:` como lista
   (ver debajo).
3. **Secuencia-raíz directa**: el YAML entero es una lista de cases,
   sin ninguna clave superior (p. ej. `narrative-005.yaml` empieza con
   `- id: ...`).

Además, en todas tres, los campos `suite:` y `spec_version:` **nunca
son obligatorios** en el POC: `parse_cases` sólo deserializa la lista
de cases.

**AN-020:** acepta **sólo** la primera forma — mapa con clave `cases:`
— y **exige** `suite:` + `spec_version:` como obligatorios. Las otras
dos formas de raíz producen `InvalidYaml`:

```yaml
# AN-020 canónico (única forma aceptada):
suite: nombre
spec_version: 1
cases:
  - id: foo
    ...

# Rechazado — secuencia-raíz al estilo narrative-005.yaml:
- id: foo
  function: identity
  args: [42]
  expected: 42

# Rechazado — acceptance.public:
acceptance:
  public:
    - id: foo
      ...

# Rechazado — mapa con cases pero sin suite o sin spec_version:
cases:
  - id: foo
    ...
```

**Justificación:** AN-020 trata al schema del runtime como fuente de
verdad única. `suite:` sirve a `SuiteReport.suite`; `spec_version:`
reserva espacio para evoluciones futuras del schema sin ambigüedad.
Las fixtures POC con raíz distinta (una secuencia-raíz o
`acceptance.public`) se adaptan añadiendo las dos líneas al principio
o se clasifican como `aplazado` en la matriz `poc-equivalence-matrix.md`.

**Sobre `acceptance.public`:** es un formato real del runner POC
(línea 345 de `main.rs`), **no** un invento del orchestrator.
Reconocer este formato en AN-020 implicaría doble parsing: usar la
clave `cases:` directamente es equivalente y evita la ambigüedad.

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
| Correcciones AN-020 patch 2026-10-19 | Dirección POC↔AN-019↔AN-020 para `import`/`function` invertida; `acceptance.public` y secuencia-raíz documentadas como formas-raíz reales del POC (ver `_reference/.../runner/src/main.rs:341-352`); `args` booleanos reclasificados como restauración tras regresión AN-019. |
| ADRs | ADR-004 (fail-explicit) |
| Destinos de roadmap para omitidos | INIT-07-02 (host state rico), RM-07 (records anidados), M4 (esclusas) |
