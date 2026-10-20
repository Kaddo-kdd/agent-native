---
id: AN-041
type: feature
status: completed
completed_at: 2026-10-19
created_at: 2026-10-19T00:00:00.000Z
title: Verificación de integridad SHA-256 en carga (run --verify)
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
  - WI-CANDIDATE-06-04-01
domains:
  - Security
  - Distribution & DX
  - Runtime & Memory
related_capabilities:
  - CLI as Primary Interface
  - Independent Runtime Execution
  - Versioned Publication & Evolution
expected_value: >-
  Habilitar la bandera `--verify` en el comando `run` del CLI de Agent-Native para validar la integridad
  del componente WebAssembly antes de su compilación o instanciación en el runtime. Permite contrastar el
  hash SHA-256 calculado sobre los bytes en bruto (raw bytes) del archivo contra un hash explícito
  (`--expected-sha256 <HEX>`) o contra el hash registrado en un manifiesto adyacente o explícito
  (`--manifest <PATH>`). Si el hash no coincide o el componente ha sido alterado, la ejecución se detiene
  de forma determinista con error explícito tipado (DomainFailure, exit code 1) sin instanciar el
  componente WebAssembly ni inicializar el motor de ejecución.
risks:
  - >-
    R-VER-01: Instanciación prematura antes de la validación. Mitigación: La verificación de integridad se
    ejecuta estrictamente sobre los bytes leídos de disco antes de invocar `engine.load_component()`.
  - >-
    R-VER-02: Formato de hash inconsistente (mayúsculas/minúsculas o espacios). Mitigación: Normalización
    canónica a minúsculas y trim antes de comparar.
dependencies:
  - AN-028
  - AN-033
source_signals:
  - 'Roadmap INIT-06-04: Verificación de integridad y procedencia en carga'
  - 'ADR-004: Estrategia de seguridad mínima transversal (fail-explicit, SHA-256 determinista)'
  - 'ADR-006: Empaquetado, firma y verificación para distribución multiplataforma'
decision_candidates: []
related_decisions:
  - ADR-004
  - ADR-006
---

# AN-041 — Verificación de integridad SHA-256 en carga (`run --verify`)

## Actor y Resultado
Un operador de CLI o sistema automatizado invoca `agent-native run --verify` suministrando un componente WASM y un hash SHA-256 esperado (vía `--expected-sha256` o vía manifiesto). El CLI calcula el digest SHA-256 de los bytes del componente antes de instanciarlo; si coincide, procede con la ejecución; si el componente ha sido alterado o el hash no coincide, el comando aborta inmediatamente con código de salida `1` (`DomainFailure`), garantizando que ningún byte no verificado llegue a ejecutarse.

## Acceptance Criteria Verificados
- **AC-01 (Flag `--verify` y `--expected-sha256`):** `agent-native run` soporta `--verify` y `--expected-sha256 <HEX>`. Si el hash calculado sobre los bytes coincide con el esperado, la invocación procede normalmente.
- **AC-02 (Fail-explicit ante mismatch):** Si el hash esperado no coincide con el digest real de los bytes del archivo, el comando falla inmediatamente con `CliExitCode::DomainFailure` (código 1), emitiendo un mensaje claro con el hash esperado vs el computado, sin llamar a `Engine::load_component`.
- **AC-03 (Verificación vía Manifiesto):** Si se provee `--manifest <PATH>` (o se detecta un `manifest.json` adyacente con la entrada del componente y no se pasa `--expected-sha256`), el hash esperado se extrae del manifiesto y se valida.
- **AC-04 (Salida estructurada JSON):** Al ejecutar con `--format json`, la salida de error o éxito incluye información estructurada sobre la verificación (`integrity_verified: true/false`, `computed_sha256`, `expected_sha256`).
- **AC-05 (Pruebas de integración automatizadas):** Suite de pruebas en `crates/cli/tests/run_verify.rs` verificando componente válido, componente alterado (1 byte cambiado), hash erróneo y manifiesto.
