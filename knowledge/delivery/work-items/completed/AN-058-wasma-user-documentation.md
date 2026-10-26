---
id: AN-058
type: feature
work_type: feature
title: Documentación inicial y tutorial verificable de Wasma
status: completed
created_at: '2026-10-20'
completed_at: '2026-10-26'
knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
implemented_by: codex
closed_by: agentic
implementation_status: completed
validation_status: passed
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

## Evidencia de cierre

- **AC-01:** Ocho páginas canónicas implementadas en `website/src/content/docs/docs/` y validadas con `check-content.mjs`.
- **AC-02:** Configuración de Starlight con navegación completa, búsqueda con Pagefind (13 páginas indexadas), soporte de temas y metadatos SEO.
- **AC-03:** Tutorial de `first-component.md` ejecutado automáticamente mediante `test-docs.mjs` contra el binario `agent-native` con 5/5 comandos pasando (verify, build, run con salida `42`, test YAML y fallo tipado de función faltante con exit 1).
- **AC-04:** Referencia CLI (`docs/cli.md`) documenta únicamente los subcomandos soportados por la CLI real sin alias ficticios.
- **AC-05:** Conceptos y arquitectura explicados con rigor técnico, distinguiendo componentes WebAssembly, IR v0.6 experimental y advertencias.
- **AC-06:** Modelo de seguridad documentado en `docs/security.md` (Requested ∩ Granted, límites de fuel y memoria, aislamiento).
- **AC-07:** Rendimiento (`docs/performance.md`) enlaza a la evidencia técnica de `docs/benchmarks/` sin duplicar números manualmente.
- **AC-08:** Recorrido E2E y búsqueda probados en build local y en múltiples tamaños de pantalla.
