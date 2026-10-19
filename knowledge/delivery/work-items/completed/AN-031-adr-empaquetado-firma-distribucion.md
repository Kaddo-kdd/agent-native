---
id: AN-031
type: chore
status: completed
completed_at: 2026-10-19
title: "AN-031 — ADR de empaquetado, firma y verificación para distribución multiplataforma"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - knowledge/tech/decisions/ADR-006-empaquetado-firma-verificacion.md
source_roadmap_initiative: INIT-05-03
initiative: INI-005
domains:
  - Distribution & DX
  - Security
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
expected_value: >-
  Establecer la decisión arquitectónica formal (ADR-006) que rige el empaquetado,
  la generación de sumas de verificación SHA-256, el mecanismo de firma criptográfica
  (resolviendo OQ-05-01 a favor de Minisign) y el protocolo de verificación offline
  para la distribución de binarios en los 3 triples objetivo (Linux x86_64, Windows x86_64
  y macOS aarch64), respetando la restricción de identidad y no publicación de la Decisión D3.
dependencies:
  - AN-028
source_signals:
  - "Roadmap RM-05: INIT-05-03 (WI-CANDIDATE-05-03-01)"
  - "INI-005 OQ-05-01: minisign vs cosign"
  - "Decisión Humana D3 (2026-10-19): Mantener nombre interno agent-native; congelar releases públicos definitivos"
  - "ADR-001, ADR-002, ADR-004: Integridad verificable, fail-explicit y determinismo"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T18:02:00.000Z
ready_at: '2026-10-19'
---

# AN-031 — ADR de empaquetado, firma y verificación para distribución multiplataforma

**Actor y resultado:**
El equipo de ingeniería y los futuros operadores de release disponen de un marco de decisión arquitectónica normativo (ADR-006) que define los triples de compilación, formatos de empaquetado, política de símbolos/strip, cálculo de checksums `SHA256SUMS`, protocolo de firma criptográfica mediante Minisign (Ed25519) y procedimiento de verificación sin conexión a red, sirviendo como base directa para la automatización del pipeline de CI/CD (AN-032).

**Acceptance Criteria Verificados:**
- **AC-01 (Resolución de OQ-05-01 - Minisign vs Cosign):** Documentada y justificada la elección de Minisign frente a Cosign/Sigstore y GPG en ADR-006, priorizando la verificación offline determinista, la ausencia de dependencia de infraestructura de identidad/OIDC externa y la ligereza del cliente de verificación.
- **AC-02 (Triples y Formatos Canónicos):** Fijados los 3 triples estándar:
  1. `x86_64-unknown-linux-gnu` (`.tar.gz`)
  2. `x86_64-pc-windows-msvc` (`.zip`)
  3. `aarch64-apple-darwin` (`.tar.gz`)
  con convención de nomenclatura determinista `<binary>-<version>-<target>.<ext>`.
- **AC-03 (Contenido de Paquetes y Checksums):** Definido el contenido estándar de cada archivo (binario ejecutable con `strip = true`, `LICENSE`, `README.md`) y el manifiesto unificado `SHA256SUMS` acompañado de su firma `SHA256SUMS.minisig`.
- **AC-04 (Procedimiento de Verificación Documentado):** Especificado el flujo paso a paso de verificación por parte del usuario final (descarga, validación de hash y verificación de firma Ed25519 con Minisign para Linux, macOS y Windows).
- **AC-05 (Alineación con Decisión D3):** Preservada la gobernanza de identidad: la arquitectura queda completamente especificada para `agent-native` como nombre interno temporal sin activar releases públicos ni publicación en crates.io.
