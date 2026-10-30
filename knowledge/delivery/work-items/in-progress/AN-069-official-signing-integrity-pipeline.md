---
id: AN-069
type: feature
status: in-progress
created_at: 2026-10-26T00:00:00.000Z
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
  - docs/**
expected_value: >-
  Configurar la generación de SHA-256 y firma Minisign en el pipeline de GitHub
  Actions con verificación de nombres definitivos de artefactos, repositorio
  migrado y custodia segura de claves.
project_state: ai-assisted
template_version: 1
ready_at: '2026-10-30'
---

# AN-069 — Firma oficial, integridad y controles del pipeline de publicación

## Contexto y problema

El workflow [`.github/workflows/release.yml`](file:///.github/workflows/release.yml) actualmente implementa un modo de validación (`validation`) con claves efímeras y una salvaguarda explícita contra la publicación desatendida (`if: false` en el job `publish`, y detección de `PLACEHOLDER` en la clave pública). Para preparar la release oficial v0.1.0 bajo la identidad definitiva de Wasma, se requiere formalizar la política de firma Minisign, garantizar la coherencia del catálogo de hashes `SHA256SUMS`, alinear los nombres canónicos de los secretos en GitHub Actions, documentar el procedimiento de verificación independiente para los usuarios y definir la política de custodia de claves previa a la publicación.

## Alcance

- Auditar y robustecer el workflow [`.github/workflows/release.yml`](file:///.github/workflows/release.yml) asegurando que los artefactos empaquetados (`wasma-${TAG}-${TARGET}`) correspondan a los nombres, versión (`0.1.0`) y compilación bloqueada (`cargo build --release --locked`) en Linux x86_64, Windows x86_64 y macOS ARM64.
- Alinear canónicamente el consumo de secretos en GitHub Actions a `RELEASE_MINISIGN_SECRET_KEY` (clave privada) y `RELEASE_MINISIGN_PASSWORD` (frase de paso), descartando nombres no estándar.
- Establecer validaciones de seguridad estrictas en el modo `official`: verificar que `confirm_publish` no sea solo declarativo (exigir `confirm_publish == 'true'`, ref de tag válida `refs/tags/v*`, ausencia de `PLACEHOLDER` en `release-signing-key.pub` y presencia de secretos).
- Garantizar que las claves privadas y contraseñas nunca aparezcan en logs, artefactos o repositorios (`set +x`, trap con `shred -u`).
- Establecer la política de custodia segura del par de claves Minisign del proyecto y el procedimiento operativo de provisión de la clave pública oficial (reemplazando el `PLACEHOLDER` de [`knowledge/tech/release-signing-key.pub`](file:///knowledge/tech/release-signing-key.pub) bajo autorización humana).
- Crear la guía de integridad y verificación de releases ([`docs/release-verification.md`](file:///docs/release-verification.md)) explicando la verificación independiente mediante `minisign -Vm SHA256SUMS -p ...` y `sha256sum -c SHA256SUMS`, distinguiéndola explícitamente de `wasma verify` (que valida especificaciones Semantic IR).
- Mantener estrictamente protegidas las salvaguardas de publicación: el job `publish` debe permanecer deshabilitado (`if: false`) durante AN-069, requiriendo autorización humana independiente para su activación en AN-076.

## Criterios de aceptación

- AC-01: El workflow de release genera un archivo `SHA256SUMS` con los hashes criptográficos de todos los paquetes binarios producidos bajo la nomenclatura definitiva de Wasma (`wasma-${TAG}-${TARGET}`).
- AC-02: Se verifica que los nombres de artefactos, metadatos del paquete y la URL del repositorio coincidan estrictamente con la identidad técnica y repositorio migrado en AN-078 y AN-079 (`https://github.com/Kaddo-kdd/wasma`).
- AC-03: Se integran formalmente los secretos canónicos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD` en el workflow de release para el modo `official`, manteniendo la generación efímera en modo `validation`.
- AC-04: Se documenta la clave pública oficial del proyecto y el procedimiento canónico de verificación de integridad y autenticidad (`minisign -Vm SHA256SUMS -p release-signing-key.pub`), distinguiéndolo de `wasma verify`.
- AC-05: El pipeline mantiene un gate estricto de control humano (`publish` job condicionado por aprobación manual / `if: false`) antes de liberar cualquier release o tag en GitHub Releases.

## Fuera de alcance

- Distribución en gestores de paquetes externos (Homebrew, Cargo crates.io, winget).
- Activación anticipada o desatendida del job `publish` en GitHub Actions.
- Creación de tags o releases oficiales en GitHub (reservado para AN-076).
- Sustitución real de la clave pública `PLACEHOLDER` antes de la provisión humana oficial.
- Modificación de infraestructura DNS o hosting del portal `wasma.org`.

## Validación

1. Verificación estática del workflow: Confirmar que `.github/workflows/release.yml` utiliza exclusivamente los secretos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD`.
2. Verificación de salvaguarda oficial: Confirmar que el intento de firma en modo `official` falla de manera controlada y segura si `confirm_publish` no es true o si `release-signing-key.pub` contiene `PLACEHOLDER`.
3. Verificación de modo validación (dry-run): Ejecución y validación del modo `validation` con generación efímera de firmas Minisign y verificación completa contra `SHA256SUMS`.
4. Verificación de nomenclatura de artefactos: Comprobar que los bundles generados siguen el patrón `wasma-${TAG}-${TARGET}.tar.gz` / `.zip`.
5. Verificación de protección de publicación: Confirmar que el job `publish` contiene `if: false` y requiere un gate de aprobación humana explícita.
6. Verificación documental: Existencia de la guía de verificación criptográfica en `docs/release-verification.md` diferenciada de `wasma verify`.

## Entregable

Pipeline de release auditado y alineado con secretos oficiales, política de custodia de claves documentada, guía de verificación criptográfica y controles de aprobación humana formalizados.
