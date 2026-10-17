---
type: adr
id: ADR-001
status: draft
date: 2026-10-17
created_from: knowledge/tech/discovery/decision-candidates.md
related_work_items:
  - AN-003
code:
  - crates/
---

# ADR-001 — Arquitectura inicial del workspace y dependencias entre módulos

## Context

El repositorio nuevo `Kaddo-kdd/agent-native` arranca con solo `LICENSE`. La POC congelada en `_reference/poc-agent-native/@26b6a18` concentra la lógica en tres crates:

- `irtool/lib.rs` (2749 LOC) — modelo IR, verifier, backend WASM y hash SHA-256 en un solo archivo.
- `orchestrator/lib.rs` (1963 LOC) — interpretación de narrativa, build, publicación atómica y SHA-256 fusionados.
- `runner/main.rs` (2278 LOC) — carga Wasmtime, host polyfill con allow-list y executor YAML.

La [matriz de extracción](../extraction-matrix.md) demuestra que estas fronteras están **marcadas internamente** por funciones públicas, pero mezcladas en el mismo crate. Reproducirlas sin fronteras limpias en el producto nuevo hereda deuda experimental y rompe la invariante "runtime no depende de LLM ni del compiler".

Esta decisión es bloqueante de INIT-01-01 (workspace) y precondición de todos los crates de RM-02/03/04.

## Options Considered

### Opción A — 5 crates desde el día uno (replicando POC + `cli`)

Pros: separación máxima; cada crate tiene API pública versionable.
Contras: 5 APIs públicas sin uso real todavía; alta probabilidad de rediseñarlas antes de M3; fragmenta el workspace sin beneficio inmediato.

### Opción B — Monolito con módulos internos

Pros: velocidad inicial; fronteras internas vía `pub(crate)`.
Contras: Rompe la invariante "runtime no depende del compiler" en el nivel de dependencia Cargo; dificulta reutilizar `runtime` desde SDK externo (INIT-05-02).

### Opción C — 3 crates (`ir`, `compiler`, `runtime`) + bin `cli`, con fronteras internas estrictas para la orquestación

Pros: respeta las tres invariantes del roadmap (runtime independiente, compiler puro, CLI sin lógica); captura las 3 responsabilidades de `orchestrator` como módulos internos (`spec::`, `build::`, `publish::`) sin crear un crate adicional sin justificación.
Contras: la orquestación vive en `cli` o en un módulo compartido; cuando las APIs estabilicen puede requerir extracción a un cuarto crate `orchestrator`.

## Decision

Se adopta la **Opción C**:

- **Workspace inicial:** `crates/ir`, `crates/compiler`, `crates/runtime`, `crates/cli` (bin).
- **Reglas de dependencia (invariantes):**
  - `ir` no depende de nada del producto (ni del LLM, ni del runtime, ni de Kaddo).
  - `compiler` depende únicamente de `ir`.
  - `runtime` depende únicamente de `ir`; **nunca** de `compiler` ni del LLM.
  - `cli` depende de los tres anteriores y no concentra lógica de dominio.
- **Orquestación (narrativa → spec → LLM → publicación):** vive inicialmente dentro de la capa de `cli` como módulos (`spec::`, `build::`, `publish::`) con fronteras `pub(crate)`. Se extrae a un crate `orchestrator` separado **sólo si** se cumple **al menos una** de estas condiciones:
  1. Un consumidor externo necesita invocar la orquestación sin la CLI.
  2. La publicación atómica (`publish::`) crece a > 500 LOC con pruebas aisladas.
  3. El adaptador LLM (`build::agent::`) requiere un contrato de versionamiento independiente del `cli`.
- **Interoperabilidad conservada:** `runtime` debe poder cargar componentes WIT-compatibles **independientes de su origen** (no sólo los generados por Agent-Native). Diseñar un runtime cerrado está prohibido por este ADR.
- **Lint CI:** una regla verifica que `runtime` no referencia `compiler` ni crates/adaptadores LLM; otra verifica que `compiler` no referencia store de artefactos ni `runtime`.

## Consequences

**Positivas:**
- Las tres invariantes del roadmap quedan expresadas en el grafo de dependencias Cargo, auditables por lint.
- INIT-01-01 puede abrir el workspace con contenido técnico claro por crate.
- INIT-04-05 (publicación) tiene desde el inicio una ubicación modular definida (`cli::publish::`), no en `compiler`.
- La extracción futura a `orchestrator` queda gobernada por criterios objetivos.

**Negativas:**
- La capa de orquestación crece dentro de `cli`, lo que puede dificultar su revisión aislada hasta la extracción.
- Dos consumidores externos del runtime pueden adelantarse al momento ideal de extraer `orchestrator`.
- Si la condición 1 se cumple antes que la 2 o la 3, hay que rediseñar la API pública de los módulos internos.

**Riesgos del roadmap mitigados:**
- R-03 (acoplamiento accidental runtime→compiler) — por lint CI.

**Seguimientos abiertos:**
- `[open]` Diseñar el lint CI concreto (`cargo deny`, `cargo tree -i`, o script dedicado) — se decide en INIT-01-01.
- `[open]` Determinar en qué ciclo se evalúa la extracción a `orchestrator` — propuesta: al cerrar RM-04.

## Related Capabilities

- Foundation & Governance (nuevo dominio candidato).
- Independent Runtime Execution.
- Component Compilation.
- CLI as Primary Interface.

## Related Work Items

- AN-003 (este ADR).
- Precondición de: INIT-01-01, INIT-02-01, INIT-02-02, INIT-03-01, INIT-04-05.
