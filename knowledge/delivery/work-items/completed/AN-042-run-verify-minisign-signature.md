---
id: AN-042
type: feature
status: completed
completed_at: 2026-10-19
created_at: 2026-10-19T00:00:00.000Z
title: Verificación de procedencia y firma criptográfica Minisign en carga (run --verify)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/cli/src/cli.rs
  - crates/cli/src/commands/run.rs
  - crates/cli/Cargo.toml
  - crates/cli/tests/run_verify.rs
source_roadmap_initiative: INIT-06-04
source_work_item_candidate:
  - WI-CANDIDATE-06-04-02
domains:
  - Security
  - Distribution & DX
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
  - Versioned Publication & Evolution
expected_value: >-
  Extender el pipeline de verificación en carga (`run --verify`) para validar la firma criptográfica
  Ed25519 en formato Minisign (según ADR-006) de forma 100% offline y determinista, utilizando la
  librería pura en Rust `minisign-verify`. Permite especificar la firma mediante `--signature <PATH>`
  (o resolver por convención `<component>.minisig`) y la clave pública mediante `--public-key <KEY_OR_PATH>`.
  Si la firma no corresponde a los bytes del componente o a la clave pública proporcionada, la ejecución
  se rechaza explícitamente (DomainFailure, exit code 1) impidiendo que código no verificado o de procedencia
  dudosa sea ejecutado en el runtime.
risks:
  - >-
    R-SIG-01: Dependencia de red o servicios externos en la verificación. Mitigación: Uso estricto de Minisign
    con clave pública provista localmente (archivo o inline base64); cero llamadas de red en tiempo de ejecución.
  - >-
    R-SIG-02: Verificación opcional o bypasseable inadvertidamente. Mitigación: Si se pasa `--public-key`
    o `--signature`, la verificación criptográfica es obligatoria y el fallo de validación es terminal.
dependencies:
  - AN-028
  - AN-031
  - AN-041
source_signals:
  - 'Roadmap INIT-06-04: Verificación de integridad y procedencia en carga'
  - 'ADR-004: Estrategia de seguridad mínima transversal'
  - 'ADR-006: Empaquetado, firma y verificación para distribución multiplataforma (Minisign Ed25519 offline)'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-006
---

# AN-042 — Verificación de procedencia y firma criptográfica Minisign en carga (`run --verify`)

## Actor y Resultado
Un operador de CLI o entorno de producción invoca `agent-native run --verify` suministrando una clave pública (`--public-key`) y opcionalmente una ruta de firma (`--signature`). El CLI valida la firma criptográfica Ed25519 en formato Minisign sobre los bytes del componente WASM usando `minisign-verify` de manera puramente local y offline. Si la firma es válida y coincide con la clave pública, la ejecución continúa; ante cualquier firma ausente requerida, clave inválida o contenido alterado, el proceso finaliza con código de error `1` (`DomainFailure`) sin cargar el código en el motor.

## Acceptance Criteria Verificados
- **AC-01 (Argumentos `--signature` y `--public-key`):** `RunArgs` incorpora `--signature <PATH>` y `--public-key <KEY_OR_PATH>`. La clave pública puede especificarse como ruta a archivo (`.pub`) o cadena base64 directa.
- **AC-02 (Resolución por convención):** Si se suministra `--public-key` (o `--verify`) y no se especifica `--signature`, el CLI busca automáticamente un archivo de firma adyacente `<component>.minisig`. Si no existe cuando se exige firma, falla explícitamente con mensaje de firma faltante.
- **AC-03 (Verificación criptográfica offline con `minisign-verify`):** La verificación de firma se realiza de forma estrictamente local mediante la crate `minisign-verify` sin conexiones a red ni dependencias de OIDC/Rekor.
- **AC-04 (Fail-explicit ante firma inválida o alterada):** Si los bytes del componente han sido manipulados post-firma, o si la firma no corresponde a la clave pública, la ejecución termina inmediatamente con `CliExitCode::DomainFailure` (código 1).
- **AC-05 (Integración combinada con SHA-256):** Es posible combinar la verificación de firma Minisign con la verificación de hash SHA-256 (AN-041); ambos controles deben satisfacerse para proceder.
- **AC-06 (Pruebas de integración automatizadas):** Suite de pruebas en `crates/cli/tests/run_verify.rs` cubriendo:
  1. Firma Minisign válida -> ejecución exitosa.
  2. Componente alterado post-firma -> rechazo inmediato.
  3. Firma corrupta o truncada -> rechazo inmediato.
  4. Clave pública no coincidente -> rechazo inmediato.
  5. Firma faltante cuando se requiere -> rechazo inmediato.
