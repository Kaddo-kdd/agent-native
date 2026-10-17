---
type: adr
id: ADR-004
status: draft
date: 2026-10-17
created_from: knowledge/tech/discovery/decision-candidates.md
related_work_items:
  - AN-003
code:
  - crates/compiler/
  - crates/runtime/
---

# ADR-004 — Estrategia de seguridad mínima transversal

## Context

El roadmap incorporó, como ajuste explícito, el principio "seguridad transversal, no una fase": RM-06 **profundiza** controles de seguridad pero no debe ser el primer momento en que se consideran. Los componentes compilados por Agent-Native se tratan como código no confiable (`business.md` §Constraints).

La POC implementa parcialmente este principio:
- `runner` tiene allow-list de host capabilities y propaga errores explícitos cuando un import no está autorizado (ver [extraction-matrix.md](../extraction-matrix.md) C-07, R-07 risk).
- `irtool` calcula SHA-256 por artefacto (`sha256_hex`).
- `orchestrator` publica atómicamente con hash.

El producto nuevo necesita que estos controles existan **desde RM-02/RM-03** y queden como invariantes del pipeline, no como opciones.

## Options Considered

### Opción A — Diferir seguridad a RM-06 (modelo original de la POC)

Pros: menor fricción inicial.
Contras: deja al runtime pre-RM-06 vulnerable a imports no listados; cualquier release intermedio es inseguro por diseño. **Rechazada** por el ajuste del roadmap.

### Opción B — Introducir controles mínimos transversales desde RM-02/RM-03

Pros: cualquier release del producto, incluyendo RC v0.1.0 (M5), es seguro por defecto; RM-06 extiende sin reinventar.
Contras: añade carga a los WIs tempranos; requiere disciplina en diseño de errores tipados.

### Opción C — Delegar seguridad al adaptador de host del consumidor

Pros: Agent-Native queda neutro.
Contras: traslada el riesgo a cada integrador; convierte "seguro por defecto" en opt-in. **Rechazada**.

## Decision

Se adopta la **Opción B** con cuatro controles mínimos obligatorios desde RM-02/RM-03:

1. **Fail-explicit en imports/capabilities faltantes.** El runtime devuelve `RuntimeError` tipado (nunca `0`, `false`, `null` ni valor por defecto) ante import no autorizado o capability no listada. Formalizado en INIT-03-04.
2. **Deny-by-default en capabilities del host.** El manifiesto declara una allow-list; cualquier capability no listada falla al cargar. Formalizado en INIT-03-05. RM-06/INIT-06-01 refina esta allow-list a un policy loader completo.
3. **Determinismo + hash SHA-256 por artefacto.** El pipeline IR→WASM produce artefactos reproducibles cross-OS; el hash se calcula canónicamente y se incluye en el manifiesto. Formalizado en INIT-02-04.
4. **Publicación atómica con preservación de versión anterior.** Un fallo de publicación nunca sobrescribe la versión activa. Formalizado en INIT-04-05.

**Invariantes transversales:**
- Ningún camino de código produce "resultado válido" ante un error de dependencia, capability o contrato. Toda ruta de error devuelve un valor explícitamente tipado.
- Los tests negativos son parte del DoD de cada WI que toca runtime o compiler, no un WI aparte.

**Qué refina RM-06 sobre este ADR:**
- Policy loader completo con DSL (INIT-06-01).
- Fuel accounting + `InstanceLimits` (INIT-06-02).
- Aislamiento multi-tenant con garantías (INIT-06-03).
- Verificación de firma (INIT-06-04).
- Fixtures de ejecución extrema (INIT-06-05).

Nada de lo que RM-06 añade debe presentarse como "ahora el runtime es seguro"; el runtime ya es seguro-por-defecto desde M3.

## Consequences

**Positivas:**
- RC v0.1.0 (M5) distribuido en RM-05 no requiere advertencias de seguridad.
- RM-06 queda como refinamiento genuino, no como reparación.
- Los `RuntimeError` tipados simplifican debugging y auditoría.

**Negativas:**
- Cada WI de runtime/compiler lleva pruebas negativas adicionales (DoD).
- El manifiesto crece desde el inicio con sección `capabilities:` obligatoria.

**Riesgos del roadmap mitigados:**
- Risk de ejecutar código no confiable sin controles.
- R-10 (afirmaciones sin medición) — parcialmente, al hacer explícito que seguridad ≠ rendimiento.

**Seguimientos abiertos:**
- `[open]` Formato exacto de la allow-list en el manifiesto (JSON vs YAML) — INIT-03-05.
- `[open]` Lista inicial de capabilities soportadas (consola, filesystem, …) — se define por bloque (RM-07 cubre stdio).

## Related Capabilities

- Host Capability Policy.
- Independent Runtime Execution.
- Versioned Publication & Evolution.

## Related Work Items

- AN-003 (este ADR).
- Precondición de: INIT-02-04, INIT-03-04, INIT-03-05, INIT-04-05, RM-06 completo.
