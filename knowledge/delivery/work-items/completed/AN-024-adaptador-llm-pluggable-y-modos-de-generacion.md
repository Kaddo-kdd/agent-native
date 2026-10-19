---
id: AN-024
type: feature
status: completed
completed_at: 2026-10-19
title: "AN-024 — Adaptador LLM pluggable y modos de generación"
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/cli/src/agent/**
  - crates/cli/src/lib.rs
  - crates/cli/tests/agent_providers.rs
source_roadmap_initiative: INIT-04-02
initiative: INI-004
domains:
  - Specification & Knowledge Ingestion
  - LLM-driven Implementation
related_capabilities:
  - LLM-driven Implementation
expected_value: >-
  Proveer un contrato de abstracción (`LlmProvider`) para desacoplar la generación de Semantic IR
  de la implementación concreta del modelo. Separar explícitamente tres modos de operación
  (`real`, `fixtures` y `synthesizer`), documentando sus garantías y previniendo que los modos
  offline deterministas se confundan con la capacidad generativa real del agente (Riesgo R-12).
dependencies:
  - AN-023
source_signals:
  - "POC crates/orchestrator/src/agent.rs"
  - "Roadmap RM-04: INIT-04-02"
  - "Riesgo R-12: Confundir offline con capacidad generativa real"
  - "ADR-001: Arquitectura inicial del workspace"
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-19T00:00:00.000Z
ready_at: '2026-10-19'
---

# AN-024 — Adaptador LLM pluggable y modos de generación

**Actor y resultado:**
Un orquestador de construcción, desarrollador o arnés de pruebas utiliza un proveedor de generación que implementa `LlmProvider` para proponer Semantic IR v0.5 a partir de un `SystemSpec`. El sistema opera de forma transparente en uno de tres modos explícitos (`real`, `fixtures`, `synthesizer`), auditando métricas y respetando la honestidad técnica de sus garantías.

**Current behavior:**
- Tras AN-023, existe el modelo tipado `SystemSpec` y el intérprete de narrativa Markdown.
- No existía una interfaz abstracta para la generación de IR ni adaptadores que permitan alternar entre LLMs reales y modos offline reproducibles.

**Target behavior:**
- Módulo `crates/cli/src/agent/` con:
  - `provider.rs`: Trait canónico `LlmProvider`, enum `GenerationMode` (`Real`, `Fixtures`, `Synthesizer`), `LlmConfig` y métricas `AgentExecutionMetrics`.
  - `fixtures.rs`: `FixtureLlmProvider` para pruebas deterministas basadas en fixtures pregrabados.
  - `synthesizer.rs`: `SynthesizerLlmProvider` basado en reglas directas sobre `SystemSpec`.
  - `real.rs`: `HttpLlmProvider` para integración HTTP estándar con modelos compatibles (`Anthropic`, `OpenAI`, etc.) mediante endpoints configurables.
  - `fallback.rs`: Orquestador con soporte de cadena de fallback y registro de métricas.
- Exposición pública en `crates/cli/src/lib.rs` como módulo `agent`.
- Pruebas automatizadas offline para `fixtures`, `synthesizer` y manejo de errores de conexión para `real`.

## Acceptance criteria

### AC-01 — Trait `LlmProvider` y Modos Explícitos
- Trait `LlmProvider` con métodos `name()`, `mode()`, `propose_ir()` y `last_execution_metrics()`.
- Enum `GenerationMode` con tres variantes explícitas: `Real`, `Fixtures`, `Synthesizer`.
- Cada implementación declara sin ambigüedad su modo de operación en sus métricas y respuestas.

### AC-02 — Modo `fixtures` (Reproducibilidad Determinista)
- `FixtureLlmProvider` permite registrar mapeos estáticos `(component_id, SystemSpec) -> Semantic IR (JSON Value)`.
- Si un componente no está registrado en el fixture, falla explícitamente (`AgentError::FixtureNotFound`).
- Ejecución 100% offline, determinista y adecuada para CI sin secretos.

### AC-03 — Modo `synthesizer` (Reglado Heurístico)
- `SynthesizerLlmProvider` traduce directamente entidades, funciones y operaciones de `SystemSpec` a la estructura sintáctica de Semantic IR v0.5 sin llamadas a APIs.
- Genera IR válido según `crates/ir::verify_ir`.

### AC-04 — Modo `real` (Proveedor HTTP)
- `HttpLlmProvider` soporta configuración vía variables de entorno (`AGENT_NATIVE_LLM_PROVIDER`, `AGENT_NATIVE_LLM_ENDPOINT`, `AGENT_NATIVE_LLM_MODEL`, `AGENT_NATIVE_LLM_KEY`, `AGENT_NATIVE_LLM_TIMEOUT`).
- Construye prompts estructurados incluyendo el esquema de Semantic IR v0.5 y diagnósticos previos de compilación para autorrecuperación.
- Manejo fail-explicit ante falta de API key o timeout.

### AC-05 — Mitigación de Riesgo R-12 y Honestidad Técnica
- La documentación del módulo y de los tipos explicita que `fixtures` y `synthesizer` son mecanismos de soporte para desarrollo y regresión offline, **no** la capacidad generativa inteligente del agente.

### AC-06 — Cobertura y Regresión Offline
- Pruebas unitarias e integración en `crates/cli/tests/agent_providers.rs` verificando los modos `fixtures`, `synthesizer`, el fallback y la validación de configuración de `real`.
- Todas las pruebas de CI corren sin requerir API keys ni conexión a internet.

## Out of scope
- Pipeline completo de build/evolución incremental (pertenece a INIT-04-03 / INIT-04-05).
- Despliegue de modelos locales (vLLM/Ollama embebidos en el workspace).

## Validation
```bash
cargo test -p agent-native
cargo test --workspace
```

## Traceability

| Elemento | Ubicación |
|---|---|
| Trait y tipos base | `crates/cli/src/agent/provider.rs` |
| Errores de agente | `crates/cli/src/agent/error.rs` |
| Modo Fixtures | `crates/cli/src/agent/fixtures.rs` |
| Modo Synthesizer | `crates/cli/src/agent/synthesizer.rs` |
| Modo Real (HTTP) | `crates/cli/src/agent/real.rs` |
| Cadena de Fallback | `crates/cli/src/agent/fallback.rs` |
| Exportación de módulo | `crates/cli/src/agent/mod.rs`, `crates/cli/src/lib.rs` |
| Pruebas de integración | `crates/cli/tests/agent_providers.rs` |

## Learning

**Lo que salió como se esperaba:**
- Desacoplamiento total: la generación de Semantic IR se abstrae completamente a través del trait `LlmProvider`.
- Cumplimiento de honestidad técnica (Riesgo R-12): la distinción canónica entre `Real`, `Fixtures` y `Synthesizer` se refleja no sólo en documentación, sino a nivel de tipos (`GenerationMode`) y en la telemetría obligatoria (`AgentExecutionMetrics`).
- Cobertura 100% offline: los arneses de prueba y CI pueden ejecutar síntesis heurística y reproducción de fixtures con cero riesgo de flakiness por red o falta de credenciales.
- La síntesis heurística produce documentos que satisfacen estrictamente `agent_native_ir::verify_ir`, generando funciones, tipos, contratos WIT y trazabilidad a requisitos.

**Lo inesperado:**
- Al encadenar `FallbackLlmProvider`, la telemetría reporta con precisión qué eslabón de la cadena resolvió la propuesta (e.g. `resilient-agent:fallback-synth`), permitiendo auditar cuándo una degradación ocurre en producción.
- La suite completa del workspace aumentó a 87 pruebas unitarias e integración en el workspace principal, más 22 pruebas en los ejemplos de AWS Lambda (109 tests en total), todas ejecutándose en milisegundos sin llamadas externas.

### Evidencias de ejecución
1. **Compilación y Linteo:**
   - `cargo clippy --workspace --all-targets`: 0 warnings, formato y linteo pedantic limpios.
2. **Pruebas de la suite `agent-native`:**
   - 3 pruebas unitarias en `crates/cli/src/spec/interpreter.rs`.
   - 4 pruebas de integración en `crates/cli/tests/spec_interpreter.rs`.
   - 5 pruebas de integración en `crates/cli/tests/agent_providers.rs`.
   - Total `agent-native`: 12 passed; 0 failed.
3. **Workspace y Ejemplos:**
   - 87 pruebas del workspace principal superadas.
   - 22 pruebas de AWS Lambda superadas (11 en runtime example + 11 en benchmark native).
   - Total global: 109 passed; 0 failed.
