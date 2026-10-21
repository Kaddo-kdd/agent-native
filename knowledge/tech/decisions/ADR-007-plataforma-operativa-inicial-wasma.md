---
type: adr
id: ADR-007
status: accepted
date: 2026-10-21
related_work_items:
  - AN-061
  - AN-062
code:
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
---

# ADR-007 — Plataforma operativa recomendada para la primera versión de Wasma

## Context

El runtime y la CLI se validan funcionalmente mediante CI en Linux, Windows y
macOS. AN-061 y AN-062 documentaron variabilidad de rendimiento en runners
hospedados; no certificaron WASI ni establecieron comparaciones de rendimiento
entre sistemas operativos. La primera versión pública necesita un entorno de
referencia claro para integraciones, ejemplos y futuras comparaciones de caché,
warm pool y concurrencia, sin retirar la portabilidad funcional existente.

## Decision

- **Linux x86_64 GNU** es la plataforma operativa recomendada para las primeras
  integraciones y despliegues controlados de Wasma, y el entorno de referencia
  para el trabajo operativo de RM-08.
- **Windows x86_64** y **macOS ARM64** permanecen compatibles y validados por la
  matriz funcional de CI. No se presentan como plataformas de referencia de
  rendimiento ni como certificación para toda carga de producción.
- Los artefactos de distribución multiplataforma definidos por ADR-006 no se
  retiran. Esta decisión prioriza ejemplos e investigación operativa; no
  reduce los triples objetivo ni modifica la validación CI.
- La decisión no certifica WASI, binarios WASI reales, rendimiento estable,
  caché, warm pool o concurrencia. Esas afirmaciones siguen sujetas a sus
  work items y evidencia correspondiente.

## Consequences

La documentación de primeras integraciones y los experimentos operativos de
RM-08 parten de Linux x86_64 GNU. La matriz Linux/Windows/macOS continúa siendo
un requisito de regresión funcional. Si se requiere una afirmación de
rendimiento entre hosts o una garantía operativa adicional para Windows o
macOS, deberá usarse un entorno controlado y registrarse una decisión nueva.

## Validation

La decisión se apoya en la CI cross-OS verde y en el diagnóstico archivado de
AN-061/AN-062. No usa las cifras de benchmark para afirmar que Linux sea más
rápido; el diagnóstico también observó variabilidad en Linux.
