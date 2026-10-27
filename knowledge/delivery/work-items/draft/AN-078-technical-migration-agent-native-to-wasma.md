---
id: AN-078
type: feature
status: draft
created_at: 2026-10-27T00:00:00.000Z
title: Migración técnica de Agent Native a Wasma
knowledge_level: K4
suggest_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-11
priority: P0
depends_on:
  - AN-077
domains:
  - Core Runtime & Tooling
  - Developer Experience, Documentation & Branding
expected_value: >-
  Alineación consistente de manifests Cargo, lockfile, CLI binaria, imports, tests y CI con la identidad
  aprobada de Wasma, preservando contratos Semantic IR/WIT, garantías de seguridad y trazabilidad.
project_state: ai-assisted
template_version: 1
---

# AN-078 — Migración técnica de Agent Native a Wasma

## Contexto y problema

La identidad pública del producto es Wasma (`wasma.org`), pero las interfaces técnicas internas y públicas todavía utilizan nombres `agent-native` (ejecutable CLI, crates Cargo, dependencias internas y rutas de módulos). Esta discrepancia dificulta la instalación para nuevos usuarios, genera confusión en la documentación y provocaría una primera release oficial con nombres heredados inconsistentes.

## Alcance

Aplicar exclusivamente los nombres aprobados en AN-077 sobre las siguientes superficies:

- `Cargo.toml`, manifests de crates (`wasma-ir`, `wasma-compiler`, `wasma-runtime`, CLI) y `Cargo.lock`.
- Ejecutable de CLI y nombre de paquete Cargo.
- Declaraciones de dependencias internas y rutas `use` afectadas en todo el workspace.
- Pruebas unitarias, de integración, regresión y E2E.
- Fixtures, scripts y ejemplos públicos.
- GitHub Actions de CI, benchmarks y release.
- README, guía de instalación, documentación web y referencias de comandos.
- Configuración Kaddo que dependa de nombres técnicos o rutas del repositorio.

## Restricciones arquitectónicas

- Preservar los contratos Semantic IR y WIT existentes sin alteración.
- No renombrar interfaces externas versionadas sin una decisión independiente.
- No cambiar la semántica del compilador ni del runtime.
- Mantener las garantías de `Store` e instancia fresca, aislamiento de tenants y políticas de seguridad.
- Mantener los workers experimentales deshabilitados por defecto.
- No reescribir los commits, ADRs, Work Items, hashes, resultados ni artefactos históricos.
- No reinicializar Kaddo ni perder su identificador o trazabilidad sin una migración expresamente aprobada.
- No realizar cambios funcionales que no sean estrictamente necesarios para la migración.

## Criterios de aceptación

- AC-01: Los nombres públicos definidos en AN-077 están implementados consistentemente en todo el workspace.
- AC-02: La CLI renombrada ejecuta los cinco comandos actuales (`compile`, `run`, `verify`, `doctor`, `inspect`) respetando sus contratos existentes.
- AC-03: Cualquier alias de compatibilidad aprobado en AN-077 se encuentra probado y documentado.
- AC-04: Los manifests de Cargo y el lockfile resuelven correctamente todas las dependencias internas sin errores de resolución.
- AC-05: Pasan `cargo fmt --check`, `cargo clippy`, compilación y suite completa de pruebas normales y experimentales en las plataformas soportadas.
- AC-06: Los tests E2E verifican que se preserva íntegramente el comportamiento de los componentes WebAssembly.
- AC-07: Documentación, tutoriales y CI utilizan los nuevos nombres, sin depender de rutas heredadas no aprobadas.
- AC-08: Las referencias históricas (ADRs, WIs previos AN-001..AN-067) permanecen intactas y consultables.
- AC-09: Se revisa y valida el diff completo antes de cualquier commit.

## Fuera de alcance

- Cambio del nombre remoto en GitHub (cubierto en AN-079).
- Configuración de cuentas externas y publicación en crates.io.
- Firma y publicación de release oficial (cubiertos en AN-069 y AN-076).

## Entregable

Migración técnica reproducible y validada en el workspace Rust, con evidencia de regresión y suite de pruebas en verde.
