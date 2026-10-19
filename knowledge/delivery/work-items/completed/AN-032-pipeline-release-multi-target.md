---
id: AN-032
type: chore
status: completed
completed_at: 2026-10-19
title: "AN-032 — Pipeline multi-target de release y firma en CI"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - .github/workflows/release.yml
  - knowledge/tech/release-signing-key.pub
  - knowledge/tech/release-verification.md
  - crates/cli/tests/release_packaging_test.rs
  - Cargo.toml
source_roadmap_initiative: INIT-05-03
initiative: INI-005
domains:
  - Distribution & DX
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
expected_value: >-
  Implementar la automatización de CI/CD en GitHub Actions (`.github/workflows/release.yml`)
  que compila la CLI standalone en los 3 triples objetivo (Linux x86_64, Windows x86_64,
  macOS aarch64), empaqueta los artefactos con LICENSE y README, genera el catálogo unificado
  SHA256SUMS, ejecuta la firma criptográfica con Minisign y valida la integridad de los paquetes,
  respetando la restricción de identidad de la Decisión D3 y eliminando fallbacks inseguros.
dependencies:
  - AN-028
  - AN-031
source_signals:
  - "Roadmap RM-05: INIT-05-03 (WI-CANDIDATE-05-03-02)"
  - "ADR-006: Empaquetado, firma y verificación para distribución multiplataforma"
  - "Decisión Humana D3 (2026-10-19): Congelar releases públicos definitivos; mantener nombre técnico interno"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T18:07:00.000Z
ready_at: '2026-10-19'
---

# AN-032 — Pipeline multi-target de release y firma en CI

**Actor y resultado:**
El equipo de ingeniería y los mantenedores del repositorio cuentan con un flujo de trabajo de GitHub Actions (`.github/workflows/release.yml`) que automatiza de extremo a extremo la compilación release de la CLI standalone `agent-native` en los tres sistemas operativos principales, empaquetando los binarios, calculando las sumas de comprobación `SHA256SUMS`, validando firmas Minisign y garantizando segregación estricta entre validación técnica y publicación formal.

**Acceptance Criteria Verificados y Endurecidos:**
- **AC-01 (Matriz de Compilación Multi-Target):** Configurada en [`.github/workflows/release.yml`] cubriendo los 3 triples canónicos:
  - `x86_64-unknown-linux-gnu` en `ubuntu-latest`
  - `x86_64-pc-windows-msvc` en `windows-latest`
  - `aarch64-apple-darwin` en `macos-latest` (con comprobación explícita de arquitectura arm64 vía `file`).
- **AC-02 (Empaquetado conforme a ADR-006):**
  - Configurado `strip = true` en `[profile.release]` de `Cargo.toml` y ejecución de `strip` en Linux/macOS sin ocultar errores.
  - Empaquetado `.tar.gz` (Linux/macOS) y `.zip` (Windows) conteniendo estrictamente en raíz el binario ejecutable, `LICENSE` y `README.md`.
  - Nomenclatura determinista `agent-native-${VERSION}-${TARGET}.${EXT}`.
- **AC-03 (Catálogo Unificado SHA256SUMS y Minisign sin Fallbacks Inseguros):**
  - Generación del catálogo `SHA256SUMS` ordenado alfabéticamente por nombre de archivo.
  - Verificación determinista de sumas vía `sha256sum -c SHA256SUMS`.
  - **Eliminación total de placeholders de firma simulados:**
    - *Modo Validación (Dry-Run / CI testing):* Genera un par de claves Minisign efímero en memoria/directorio seguro (`chmod 700`), firma el catálogo como `SHA256SUMS.test-minisig` con comentario de prueba, verifica la firma y destruye la clave privada.
    - *Modo Distribución Oficial:* Falla inmediatamente si faltan secretos o si `release-signing-key.pub` es un placeholder.
- **AC-04 (Documentación de Verificación y Estado de Clave):**
  - Guía [`knowledge/tech/release-verification.md`] distinguiendo las tres garantías: integridad (SHA-256), autenticidad (Minisign) y reproducibilidad.
  - Archivo [`knowledge/tech/release-signing-key.pub`] documentado como plantilla/placeholder pendiente de aprovisionamiento por el mantenedor humano (bloqueante para la firma oficial).
- **AC-05 (Cumplimiento Estricto de Decisión D3 y Mínimo Privilegio):**
  - Permisos top-level y de jobs de compilación/validación fijados en `contents: read`.
  - Job `publish` a GitHub Releases completamente desactivado bajo `if: false`, impidiendo publicaciones accidentales ante `git push` de tags.
  - Cero publicaciones en crates.io y preservación del nombre interno `agent-native`.
- **AC-06 (Suite de Pruebas Automatizadas de Release):**
  - Implementada en [`crates/cli/tests/release_packaging_test.rs`], validando generación de catálogo, detección de manipulaciones (tamper detection), contrato de archivos empaquetados e invariantes de seguridad del workflow de GitHub Actions.

**Evidencia de Verificación:**
- `cargo build --release -p agent-native --bin agent-native`: exitoso (`target/release/agent-native.exe` generado y probado con `strip = true`).
- `.\target\release\agent-native.exe --help`: ejecuta los 5 subcomandos canónicos y opciones `--format human|json`.
- `cargo test --test release_packaging_test`: 4 passed, 0 failed.
- `cargo test --workspace`: 129 passed, 0 failed.
- `cargo clippy --all-targets --workspace -- -D warnings`: 0 advertencias.
- `cargo fmt --check`: limpio.
