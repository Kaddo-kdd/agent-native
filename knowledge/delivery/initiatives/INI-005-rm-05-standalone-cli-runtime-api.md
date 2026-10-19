---
type: initiative
id: INI-005
title: RM-05 — Standalone CLI & Runtime API
status: completed
completed_at: '2026-10-19'
knowledge_level: K3
domains:
  - Distribution & DX
  - Runtime & Memory
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
roadmap_block: RM-05
roadmap_initiatives:
  - INIT-05-01
  - INIT-05-02
  - INIT-05-03
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-05-01
    title: Bin `cli` con build/verify/test/run/publish
    status: completed
    materialized_as: AN-028
  - id: INIT-05-02
    title: API pública reutilizable del runtime
    status: completed
    materialized_as: AN-029, AN-030
  - id: INIT-05-03
    title: Pipeline de release para 3 triples objetivo
    status: completed
    materialized_as: AN-031, AN-032
---

# RM-05 — Standalone CLI & Runtime API

## Goal

Convertir el pipeline interno en herramienta distribuible: un desarrollador en una máquina sin Rust ni acceso a LLM puede instalar la CLI, descargar un componente publicado y ejecutarlo con una suite de aceptación.

## Expected Value

Primer **hito público** del producto (RC v0.1.0 candidato). Materializa la invariante "ejecución sin toolchain" documentada en `business.md`.

## Scope

- Bin `cli` con subcomandos `build`, `verify`, `test`, `run`, `publish`.
- Salidas estructuradas opcionales (`--format=json`).
- API pública curada del `runtime` para embeber desde otras apps.
- Pipeline de release para `x86_64-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`.
- ADR dedicada para empaquetado, firma, publicación y verificación.

## Out of Scope

- OCI, Homebrew, winget, apt (diferidos).
- Imagen Docker oficial (diferida).
- SDKs para lenguajes no-Rust (Not Now).

## Success Criteria

1. `agent-native --help` lista los 5 subcomandos y sus opciones estables.
2. Smoke E2E en 3 OS: instalación, descarga, ejecución con resultado verificable.
3. Suites automatizadas de CI no parsean texto humano del CLI (consumen `--format=json`).
4. Ejemplo `embed_runtime.rs` compila y ejecuta un componente publicado desde Rust externo.
5. ADR de empaquetado/firma cerrada; artefactos firmados descargables con verificación documentada.

## Dependencies

- INIT-03-03 (executor YAML) e INIT-04-05 (publicación) completados.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

- OQ-05-01: minisign vs cosign — resuelta en INIT-05-03 a favor de Minisign (ADR-006).
- OQ-05-02: política de versionado semántico de la CLI vs runtime vs IR — a definir.

## Learning

1. **CLI como Contrato Estable (INIT-05-01 / AN-028):**
   La separación estricta entre el parser de comandos (`clap`) y la lógica de dominio en módulos especializados (`build`, `verify`, `test`, `run`, `publish`) garantizó que tanto el formato `--format=human` como `--format=json` operen sin fugas de texto ni fallbacks silenciosos, asegurando que las suites CI automatizadas puedan parsear resultados JSON de forma determinista.

2. **API Pública del Runtime y Aislamiento (INIT-05-02 / AN-029, AN-030):**
   La decisión de no inventar abstracciones artificiales (preservando `Engine`, `Component`, `HostPolicy`, `RuntimeError`, `Val` y `Resolver`) permitió estabilizar una superficie limpia de rustdoc y verificar que aplicaciones externas puedan embeber el runtime sin depender del compilador ni de LLMs. El ejemplo canónico `crates/runtime/examples/embed_runtime.rs` demostró la efectividad de la política deny-by-default y la limpieza de sesión vía `Resolver::reset()`.

3. **Arquitectura de Release y Verificación Offline (INIT-05-03 / AN-031, AN-032):**
   La adopción de Minisign (Ed25519) en ADR-006 resolvió OQ-05-01 proporcionando verificación criptográfica completamente desconectada (sin dependencia de infraestructura OIDC/Keyless). El flujo en GitHub Actions automatiza la compilación release con símbolos depurados (`strip = true`), empaquetado para los 3 triples objetivo (Linux x86_64, Windows x86_64, macOS aarch64), generación del manifiesto `SHA256SUMS` y firma.

4. **Gobernanza de Identidad (Decisión D3):**
   La arquitectura técnica y los flujos de CI quedan 100% operativos bajo el identificador interno temporal `agent-native`. Se respeta estrictamente la congelación de releases públicos en crates.io y anuncios de branding hasta recibir la resolución humana definitiva.

5. **Estado de Distribución y Gobernanza (Estabilización AN-032):**
   Se distingue formalmente entre:
   - *Pipeline implementado y validado funcionalmente:* 100% completado (matriz de 3 triples, empaquetado, SHA256SUMS, verificación con clave efímera de prueba y suite de regresión).
   - *Firma oficial disponible:* Pendiente de que el mantenedor humano genere y configure el par de claves Minisign legítimo en el repositorio y GitHub Secrets (actualmente `release-signing-key.pub` es un placeholder).
   - *Release público autorizado:* Congelado hasta que la Decisión D3 sea formalmente levantada tras la definición de la nueva identidad.
   - *Distribución firmada verificada end-to-end:* Se ejecutará cuando los dos puntos anteriores estén satisfechos.
