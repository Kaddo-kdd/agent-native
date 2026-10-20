---
id: AN-058
type: feature
work_type: feature
title: Documentación inicial y tutorial verificable de Wasma
status: draft
created_at: '2026-10-20'
knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
domains:
  - Developer Experience, Documentation & Branding
depends_on:
  - AN-057
code:
  - website/src/content/docs/**
  - website/scripts/**
  - website/validation/**
---

# AN-058 — Documentación inicial y tutorial verificable de Wasma

## Actor and outcome

Un desarrollador instala la CLI, produce y ejecuta su primer componente y comprende
sus límites de seguridad sin depender de conocimientos internos del repositorio.
Derivado de [AN-056](AN-056-sitio-oficial-y-documentacion-wasma.md).

## Current behavior / Target behavior / Problem

README y guía contienen discrepancias con `crates/cli/src/cli.rs`. Crear contenido
seleccionado en español con pruebas de los comandos, sin trasladar errores ni publicar
documentos internos indiscriminadamente.

## Entry points / End-to-end flow

`/docs/` → instalación → primer componente → copiar comandos → salida comprobable →
referencia CLI y seguridad; búsqueda y navegación entre documentos permiten regresar.

## Scope / Acceptance criteria

- AC-01: páginas `/docs/`, `/docs/installation/`, `/docs/first-component/`,
  `/docs/concepts/`, `/docs/cli/`, `/docs/security/`; además `/docs/runtime/` y
  `/docs/performance/`. El tutorial cubre Examples; no crear categorías vacías.
- AC-02: cada página incluye título/descripción; Starlight ofrece sidebar, TOC,
  búsqueda, resaltado, copia, temas, edición GitHub y navegación anterior/siguiente.
  Español inicial; configuración que permita inglés posterior sin duplicación ahora.
- AC-03: tutorial con archivos completos, directorio de ejecución, instalación explícita,
  versión/SHA de CLI y salida esperada. Smoke automatizado ejecuta los mismos bloques
  publicados, offline y sin claves LLM, verifica salida y un error documentado.
- AC-04: referencia cubre los cinco subcomandos reales y opciones verificadas;
  `build <input> --output ...`, `verify <input>` y `run --verify` se distinguen.
  No publicar `spec`, `wasma init` o `build --spec` como comandos disponibles.
- AC-05: explicar SystemSpec, IR, WIT, WASM, Component Model, runtime sin LLM,
  modos de generación y limitaciones. v0.6 se identifica experimental; WASI baseline
  no promete la ampliación pendiente AN-050. Nota sobre ejecutable `agent-native`.
- AC-06: seguridad explica permisos, fuel, memoria guest frente a host e integridad;
  ejemplos y garantías se contrastan con implementación y pruebas existentes.
- AC-07: performance advierte “candidata, no certificada”, enlaza
  `docs/benchmarks/README.md` y evidencia; no copia cifras manualmente.
- AC-08: E2E instalación → tutorial → resultado → seguridad funciona desde checkout
  limpio y desde navegación móvil; búsqueda encuentra la guía de instalación en build.

## Impact analysis / Module coverage

Contenido, configuración documental y scripts de validación web: affected.
`crates/*`, `knowledge/tech/` y `docs/benchmarks/`: reviewed-not-affected como código;
fuentes de lectura para documentación. Backend/DB/auth/analytics: not-applicable.

## Open Questions / Dependencies / Scope confidence

Depende de AN-057. WEB-04: elegir instalación/versionado y comprobar tutorial antes
de cerrar. K3 y confianza medium porque los ejemplos actuales no son evidencia de
ejecución. Fuentes: parser CLI, `cli_e2e.rs`, `run_limits.rs`, `run_verify.rs` y ADRs.

## Out of scope

Cambiar CLI para aceptar ejemplos erróneos, fijar APIs futuras, completar AN-050 o
certificar AN-055; corrección general de documentación histórica fuera del sitio.

## How to test it / Definition of Done

Implementar y ejecutar `pnpm --dir website test:docs`; registrar herramientas,
versiones, salida y códigos de error. Ejecutar check/build/check:links de AN-060 al
integrar. Probar búsqueda sobre build, copia y enlace GitHub de edición. DoD: todos
los AC con evidencia y contenido revisado; no afirmar pruebas que aún no se ejecutaron.
Las aprobaciones y exclusiones de AN-056 siguen aplicando.
