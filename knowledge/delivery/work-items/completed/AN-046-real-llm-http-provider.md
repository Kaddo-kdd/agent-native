---
id: AN-046
type: feature
status: completed
created_at: 2026-10-20T00:35:00.000Z
completed_at: 2026-10-20T01:40:00.000Z
title: Integración HTTP real del proveedor LLM (HttpLlmProvider)
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/cli/src/agent/real.rs
  - crates/cli/src/agent/provider.rs
  - crates/cli/src/agent/error.rs
  - crates/cli/Cargo.toml
  - crates/cli/tests/agent_real_http.rs
source_roadmap_initiative: INIT-04-02
initiative: INI-004
domains:
  - Specification & Knowledge Ingestion
  - LLM-driven Implementation
  - Distribution & DX
related_capabilities:
  - LLM-driven Implementation
  - CLI as Primary Interface
expected_value: >-
  Reemplazar el placeholder de `HttpLlmProvider::propose_ir` por una integración HTTP real que consuma
  endpoints externos (Anthropic por defecto y OpenAI como adaptador adicional) para generar Semantic IR v0.5
  a partir de SystemSpec. La integración valida estructuralmente el IR con `agent_native_ir::verify_ir`,
  registra métricas reales de duración y tokens, previene fugas de credenciales en logs y reportes,
  y habilita `agent-native build --mode real` y `agent-native publish --mode real` sin alterar el runtime.
risks:
  - >-
    R-LLM-01: Fuga de credenciales o API keys en mensajes de error o logs. Mitigación: Sanitización
    estricta de errores; las cabeceras de autorización y el contenido de `AGENT_NATIVE_LLM_KEY` jamás se
    imprimen ni se incluyen en `AgentError` ni en `InvocationReport`.
  - >-
    R-LLM-02: Dependencia de servicios externos en CI o ejecuciones offline. Mitigación: Las suites de
    prueba automáticas utilizan servidores HTTP mock locales en proceso (`std::net::TcpListener`),
    garantizando ejecución 100% offline y determinista.
  - >-
    R-LLM-03: Respuestas malformadas o alucinadas del LLM que rompan el compilador. Mitigación: Extracción
    robusta de bloques JSON (con o sin markdown fences) y validación inmediata con `agent_native_ir::verify_ir()`
    antes de entregar el IR al pipeline de construcción.
  - >-
    R-LLM-04: Violación de aislamiento arquitectónico (ADR-002). Mitigación: Las dependencias de transporte
    HTTP (`ureq`) residen exclusivamente en `crates/cli`; `crates/runtime` e `ir` permanecen libres de HTTP y LLM.
dependencies:
  - AN-023
  - AN-024
  - AN-028
source_signals:
  - "Roadmap RM-04: INIT-04-02 Adaptador LLM pluggable y modos de generación"
  - "Riesgo R-12: Confundir offline / heurística con capacidad generativa real"
  - "ADR-001: Separación de responsabilidades entre crates"
  - "ADR-002: Runtime independiente del compilador y de LLMs"
  - "ADR-004: Seguridad fail-explicit y ausencia de fallbacks silenciosos"
---

# AN-046 — Integración HTTP real del proveedor LLM (HttpLlmProvider)

## Contexto y Motivación

En `crates/cli/src/agent/real.rs`, la implementación de `HttpLlmProvider::propose_ir()` contenía un placeholder que devolvía `AgentError::Http("Endpoint ... connection not established in offline test environment")`. Aunque existían `GenerationMode::Real`, `LlmConfig` y las variables de entorno asociadas, el modo `real` no podía generar Semantic IR conectándose efectivamente a un modelo externo.

El modo `synthesizer` provee una traducción heurística determinista para pruebas y desarrollo offline, pero no debe confundirse con generación asistida por modelos (Riesgo R-12). Esta pieza completa la capacidad generativa real de Agent Native.

## Alcance Funcional Entregado

1. **Cliente HTTP Síncrono y Ligero:**
   - Incorporación de `ureq = { version = "2.10", features = ["json"] }` en `crates/cli/Cargo.toml` con TLS basado en `rustls` (fijado deterministamente para compatibilidad con Rust 1.85).
2. **Soporte de Proveedores y Formatos de Wire:**
   - **Anthropic (default):** Endpoint `https://api.anthropic.com/v1/messages`, headers `x-api-key` y `anthropic-version: 2023-06-01`, payload con `model`, `max_tokens`, `system`, y `messages: [{ role: "user", content: ... }]`.
   - **OpenAI:** Endpoint `/v1/chat/completions`, header `Authorization: Bearer <key>`, payload con `model`, `messages: [{ role: "system", ... }, { role: "user", ... }]`.
3. **Construcción de Prompts:**
   - Reutilización de `HttpLlmProvider::build_system_prompt()` y `HttpLlmProvider::build_user_prompt(component_id, spec, diagnostics)`.
4. **Extracción y Validación de Semantic IR v0.5:**
   - Soporte para desempacar JSON puro o envuelto en bloques markdown (````json ... ````).
   - Validación estructural inmediata mediante `agent_native_ir::verify_ir(&ir_value)`. Si la verificación arroja diagnósticos de error (`E*`), retorna `AgentError::InvalidIr`.
5. **Métricas de Telemetría:**
   - Medición precisa de tiempo de pared (`Instant::now().elapsed()`).
   - Extracción de `prompt_tokens` y `completion_tokens` del bloque de uso del proveedor (`usage`).
6. **Manejo de Errores y Seguridad:**
   - Error previo si `AGENT_NATIVE_LLM_KEY` falta antes de realizar la conexión (`AgentError::MissingApiKey`).
   - Mapeo de errores HTTP 401/403 a fallos claros de autenticación sin registrar la clave.
   - Timeouts configurables vía `AGENT_NATIVE_LLM_TIMEOUT` (default: 30 segundos).
   - Límite de tamaño de lectura de respuesta (10 MB).
   - Prohibición de fallback silencioso a `synthesizer` cuando falla `real` (ADR-004).

## Verificación y Pruebas

La suite [`crates/cli/tests/agent_real_http.rs`](crates/cli/tests/agent_real_http.rs) valida 100% offline mediante un servidor HTTP mock en proceso (`std::net::TcpListener`):
- `test_http_provider_missing_api_key_fails_fast`: Falla previa sin realizar conexiones cuando falta la clave.
- `test_http_provider_anthropic_wire_format_success`: Envío y parseo exitoso con formato Anthropic v1/messages y extracción de tokens.
- `test_http_provider_openai_wire_format_success`: Envío y parseo exitoso con formato OpenAI chat/completions y métricas.
- `test_http_provider_unwraps_markdown_code_fences`: Limpieza automática de delimitadores markdown code blocks.
- `test_http_provider_rejects_invalid_json_body`: Rechazo tipado con `AgentError::Serialization` ante cuerpos no JSON.
- `test_http_provider_rejects_invalid_semantic_ir_schema`: Rechazo tipado con `AgentError::InvalidIr` al violar reglas de esquema v0.5.
- `test_http_provider_handles_http_401_without_key_leak`: Manejo seguro de errores 401 sin fugar la clave en logs ni errores.
- `test_http_provider_handles_http_500_server_error`: Manejo de errores 500 del servidor.
- `test_cli_build_mode_real_e2e_against_mock`: Prueba E2E de `agent-native build --mode real` contra mock HTTP local.
