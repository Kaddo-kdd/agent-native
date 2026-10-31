---
id: AN-069
type: feature
status: completed
created_at: 2026-10-26T00:00:00.000Z
ready_at: '2026-10-30'
completed_at: '2026-10-31'
title: 'Firma oficial, integridad y controles del pipeline de publicación'
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-02
priority: P0
depends_on:
  - AN-068
domains:
  - Distribution & Release
  - Security
code:
  - .github/workflows/release.yml
  - knowledge/tech/release-signing-key.pub
  - crates/cli/tests/release_packaging_test.rs
  - docs/release-verification.md
expected_value: >-
  Configurar la generación de SHA-256 y firma Minisign en el pipeline de GitHub
  Actions con verificación de nombres definitivos de artefactos, repositorio
  migrado y custodia segura de claves.
project_state: ai-assisted
template_version: 1
---

# AN-069 — Firma oficial, integridad y controles del pipeline de publicación

## Contexto y problema

El workflow [`.github/workflows/release.yml`](file:///.github/workflows/release.yml) implementaba originalmente un modo de validación (`validation`) con claves efímeras y una salvaguarda básica contra la publicación desatendida (`if: false` en el job `publish`, y detección de `PLACEHOLDER` en la clave pública). Para preparar la release oficial v0.1.0 bajo la identidad definitiva de Wasma, se requería formalizar la política de firma Minisign, garantizar la coherencia del catálogo de hashes `SHA256SUMS`, alinear los nombres canónicos de los secretos en GitHub Actions, documentar el procedimiento de verificación independiente para los usuarios, segregar estrictamente la firma oficial detrás de un gate de aprobación humana y definir la política de custodia de claves previa a la publicación.

## Nota de Decisión: Delimitación de alcance técnico y operativo (AN-069 vs AN-076)

> [!IMPORTANT]
> **Decisión de Delimitación Aprobada por el Usuario (2026-10-31):**
> 
> - **Definición original de AC-03 y AC-04:** En el diseño inicial del Work Item, AC-03 planteaba la integración de secretos y AC-04 la documentación de la clave pública, formulados de modo que inducían a confusión entre la construcción de la infraestructura del pipeline y la provisión operativa de claves criptográficas de producción.
> - **Justificación de la delimitación:** Por principios estrictos de seguridad de la cadena de suministro de software y custodia de secretos, un asistente o agente automatizado en CI **nunca** debe generar, almacenar ni recibir claves privadas de producción. Asimismo, la configuración del GitHub Environment protegido, la asignación de Required Reviewers y el reemplazo auditado de la clave pública oficial requieren intervención humana directa.
> - **Alcance validado en AN-069:** Queda demostrada y verificada al 100 % la **infraestructura técnica**: segregación de 5 jobs (`pre-flight`, `build`, `validate`, `sign-official`, `publish`), control fail-closed ante solicitudes de `mode: official`, aislamiento absoluto de secretos, prueba end-to-end con par de claves Minisign efímero en modo `validation`, y documentación técnica canónica del procedimiento de verificación y mitigaciones de persistencia flash en SSDs (`tmpfs` en RAM).
> - **Declaración explícita de no ejecución:** En AN-069 **no** se han generado claves privadas reales, **no** se han configurado secretos en GitHub, **no** se ha habilitado operativamente el Environment `release`, **no** se ha reemplazado el valor `PLACEHOLDER` de `release-signing-key.pub`, ni se ha emitido una firma oficial de producción.
> - **Traslado operativo a AN-076:** Todas las actividades de aprovisionamiento de claves de producción, configuración del Environment en GitHub, reglas de protección de tags y verificación de firma oficial auténtica quedan transferidas formalmente como prerrequisitos obligatorios de [`AN-076 — Publicación oficial v0.1.0 y cierre de INI-009`](file:///knowledge/delivery/work-items/draft/AN-076-v010-release-publication-closure.md).

## Alcance implementado

- Auditar y robustecer el workflow [`.github/workflows/release.yml`](file:///.github/workflows/release.yml) asegurando que los artefactos empaquetados (`wasma-${TAG}-${TARGET}`) correspondan a los nombres, versión (`0.1.0`) y compilación bloqueada (`cargo build --release --locked`) en Linux x86_64, Windows x86_64 y macOS ARM64.
- Implementar una arquitectura segregada de cinco jobs:
  1. `pre-flight`: Control preventivo fail-closed que rechaza con error inmediato y código 1 cualquier ejecución en `mode: official` mientras la firma oficial permanezca deshabilitada.
  2. `build`: Compilación matricial multiplataforma, normalización de permisos Unix (0755/0644), empaquetado reproducible y eliminación garantizada de directorios temporales de staging.
  3. `validate`: Inspección estricta de inventario en `dist/` (rechaza directorios, enlaces simbólicos y archivos no esperados), generación y verificación determinista de `SHA256SUMS`, firma y verificación criptográfica end-to-end con claves efímeras Minisign, y carga del artefacto `release-bundle`.
  4. `sign-official`: Job aislado asociado al Environment protegido `release`, bloqueado con `if: false` hasta la provisión humana en AN-076.
  5. `publish`: Job de publicación formal en GitHub Releases, bloqueado estrictamente con `if: false` conforme a la Decisión D3.
- Alinear canónicamente el consumo de secretos en GitHub Actions a `RELEASE_MINISIGN_SECRET_KEY` (clave privada) y `RELEASE_MINISIGN_PASSWORD` (frase de paso), aislados exclusivamente en `sign-official`.
- Garantizar aislamiento total de secretos: los jobs `build` y `validate` no tienen acceso a secretos oficiales.
- Redactar la guía canónica de verificación de integridad y autenticidad en [`docs/release-verification.md`](file:///docs/release-verification.md), distinguiendo explícitamente `minisign -Vm SHA256SUMS -p ...` de `wasma verify`, y detallando mitigaciones de retención flash en SSDs (vía `tmpfs` en RAM) y respaldo en frío offline.
- Crear una exhaustiva suite de pruebas automatizadas en [`crates/cli/tests/release_packaging_test.rs`](file:///crates/cli/tests/release_packaging_test.rs) con 9 pruebas que validan todas las invariantes de seguridad, empaquetado, inventario, control fail-closed y compatibilidad multiplataforma de finales de línea (LF y CRLF).

## Criterios de aceptación

- AC-01: El workflow de release genera un archivo `SHA256SUMS` con los hashes criptográficos de todos los paquetes binarios producidos bajo la nomenclatura definitiva de Wasma (`wasma-${TAG}-${TARGET}`).
- AC-02: Se verifica que los nombres de artefactos, metadatos del paquete y la URL del repositorio coincidan estrictamente con la identidad técnica y repositorio migrado en AN-078 y AN-079 (`https://github.com/Kaddo-kdd/wasma`).
- AC-03 (Infraestructura de firma y aislamiento de secretos): Se implementa la arquitectura de segregación de secretos canónicos (`RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD`) aislados estrictamente en el job `sign-official` bajo el Environment `release` con `if: false` y pre-flight gate fail-closed. El modo `validation` opera de forma 100 % aislada con claves efímeras sin acceso a secretos oficiales. (La provisión y custodia de secretos oficiales queda transferida a AN-076).
- AC-04 (Procedimiento canónico de verificación e integridad): Se documenta en [`docs/release-verification.md`](file:///docs/release-verification.md) la verificación independiente con Minisign diferenciada de `wasma verify`, mitigación de retención flash en SSDs (`tmpfs` en RAM) y directrices de custodia física en frío. La clave pública se mantiene como `PLACEHOLDER` auditable hasta su sustitución humana en AN-076.
- AC-05: El pipeline mantiene un gate estricto de control humano (`publish` job condicionado por aprobación manual / `if: false` por Decisión D3 y `sign-official` bloqueado por `if: false`) antes de liberar cualquier release o tag en GitHub Releases.

## Fuera de alcance

- Distribución en gestores de paquetes externos (Homebrew, Cargo crates.io, winget).
- Activación anticipada o desatendida de los jobs `sign-official` y `publish` en GitHub Actions.
- Creación de tags o releases oficiales en GitHub (reservado para AN-076).
- Sustitución real de la clave pública `PLACEHOLDER` antes de la provisión humana oficial (reservado para AN-076).
- Generación, custodia y provisión de secretos privados de producción en GitHub Environment (responsabilidad humana exclusiva en AN-076).
- Modificación de infraestructura DNS o hosting del portal `wasma.org`.

## Evidencia de validación técnica

1. **Commit de referencia**: [`a1468c03141481cc9075ea05f8c9e4d254fcfb92`](https://github.com/Kaddo-kdd/wasma/commit/a1468c0) (`fix(test): normalize workflow line endings across platforms (AN-069)`).
2. **Workflow Website**: Run [`41213193715`](https://github.com/Kaddo-kdd/wasma/actions/runs/41213193715) — Exitoso (Green, 1m 46s).
3. **Workflow CI Multiplataforma**: Run [`41213193821`](https://github.com/Kaddo-kdd/wasma/actions/runs/41213193821) — Exitoso (Green, 12m 27s). Superó compilación, formateo, clippy y tests en Linux x86_64, Windows x86_64 y macOS ARM64.
4. **Workflow Release Multi-Platform**: Run [`41214217320`](https://github.com/Kaddo-kdd/wasma/actions/runs/41214217320) sobre `main` en `mode: validation`:
   - `pre-flight`: Exitoso en 4s (control fail-closed verificado).
   - `build` Linux (`x86_64-unknown-linux-gnu`): Exitoso en 1m 39s (`wasma-v0.1.0-dev-a1468c0-x86_64-unknown-linux-gnu.tar.gz`).
   - `build` macOS ARM64 (`aarch64-apple-darwin`): Exitoso en 2m 18s (`wasma-v0.1.0-dev-a1468c0-aarch64-apple-darwin.tar.gz`).
   - `build` Windows (`x86_64-pc-windows-msvc`): Exitoso en 3m 24s (`wasma-v0.1.0-dev-a1468c0-x86_64-pc-windows-msvc.zip`).
   - `validate`: Exitoso en 26s. Inventario exacto verificado (3 archivos regulares en `dist/`, sin carpetas temporales de staging ni symlinks). `SHA256SUMS` ordenado determinísticamente y verificado con `sha256sum -c: OK`. Firma efímera Minisign generada y verificada (`Signature and comment signature verified`). CI artifact `release-bundle` subido exitosamente.
   - `sign-official`: Omitido (`skipped`, 0s, `if: false`). Cero secretos consultados ni expuestos.
   - `publish`: Omitido (`skipped`, 0s, `if: false`). Cero tags o releases creados.
5. **Suite de pruebas de packaging local y CI**: 9 pruebas automatizadas en `crates/cli/tests/release_packaging_test.rs` ejecutadas y superadas al 100 %.
6. **Integridad de documentación**: Guía canónica [`docs/release-verification.md`](file:///docs/release-verification.md) completada.

## Entregable

Pipeline de release segregado y auditado con control fail-closed, aislamiento estricto de secretos, catálogo `SHA256SUMS` determinista, validación criptográfica efímera end-to-end, guía técnica de verificación y mitigaciones de seguridad documentadas, y transferencia formal de la provisión operativa de claves a AN-076.
