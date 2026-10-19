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
  respetando la restricción de identidad de la Decisión D3.
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
El equipo de ingeniería y los mantenedores del repositorio cuentan con un flujo de trabajo de GitHub Actions (`.github/workflows/release.yml`) que automatiza de extremo a extremo la compilación release de la CLI standalone `agent-native` en los tres sistemas operativos principales, empaquetando los binarios, calculando las sumas de comprobación `SHA256SUMS`, firmando el catálogo con Minisign y publicando los artefactos de forma reproducible.

**Acceptance Criteria Verificados:**
- **AC-01 (Matriz de Compilación Multi-Target):** Configurada en [`.github/workflows/release.yml`] cubriendo los 3 triples canónicos:
  - `x86_64-unknown-linux-gnu` en `ubuntu-latest`
  - `x86_64-pc-windows-msvc` en `windows-latest`
  - `aarch64-apple-darwin` en `macos-latest`
- **AC-02 (Empaquetado conforme a ADR-006):**
  - Configurado `strip = true` en `[profile.release]` de `Cargo.toml`.
  - Empaquetado `.tar.gz` (Linux/macOS) y `.zip` (Windows) incluyendo binario, `LICENSE` y `README.md`.
  - Nomenclatura determinista `agent-native-${VERSION}-${TARGET}.${EXT}`.
- **AC-03 (Catálogo Unificado SHA256SUMS y Firma Minisign):**
  - Job `finalize` que unifica los paquetes descargados de la matriz.
  - Generación de catálogo `SHA256SUMS` ordenado.
  - Firma Minisign (`SHA256SUMS.minisig`) con soporte de claves secretas y fallback graceful para dry-runs.
  - Verificación determinista de sumas vía `sha256sum -c SHA256SUMS`.
- **AC-04 (Documentación de Verificación):** Documento normativo [`knowledge/tech/release-verification.md`] y clave pública oficial en [`knowledge/tech/release-signing-key.pub`].
- **AC-05 (Cumplimiento de Decisión D3):** Soporta ejecución manual (`workflow_dispatch`), no publica en crates.io y mantiene la gobernanza de identidad temporal sin releases comerciales anticipados.

**Evidencia de Verificación:**
- `cargo build --release -p agent-native --bin agent-native`: exitoso (`target/release/agent-native.exe` generado y funcional con `strip = true`).
- `.\target\release\agent-native.exe --help`: ejecuta los 5 subcomandos canónicos y opciones `--format human|json`.
- Validación sintáctica de `.github/workflows/release.yml` completa.
- `cargo test --workspace`: 125 passed, 0 failed.
- `cargo clippy --all-targets --workspace`: 0 advertencias.
- `cargo fmt --check`: limpio.
