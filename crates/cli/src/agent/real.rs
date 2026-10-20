//! Pluggable HTTP LLM provider for external AI model generation.
//!
//! # Risk R-12 Notice
//!
//! This provider operates in [`GenerationMode::Real`]. It connects to an external
//! LLM endpoint (e.g. Anthropic, `OpenAI`, or compatible proxy) to generate Semantic
//! IR v0.5 proposals from specification prompts.

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use agent_native_ir::Diagnostic;
use serde_json::Value;

use crate::agent::error::AgentError;
use crate::agent::provider::{AgentExecutionMetrics, GenerationMode, LlmConfig, LlmProvider};
use crate::spec::model::SystemSpec;

/// Maximum response body length in bytes (10 MiB) to prevent memory exhaustion.
const MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

/// HTTP LLM provider for generative IR synthesis.
#[derive(Debug, Clone)]
pub struct HttpLlmProvider {
    config: LlmConfig,
    last_metrics: Arc<Mutex<Option<AgentExecutionMetrics>>>,
}

impl HttpLlmProvider {
    /// Creates a new HTTP provider with the given configuration.
    #[must_use]
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            last_metrics: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a provider loaded from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(LlmConfig::from_env())
    }

    /// Access the provider configuration.
    #[must_use]
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Builds the structured system prompt instructing the model on Semantic IR v0.5.
    #[must_use]
    pub fn build_system_prompt() -> &'static str {
        "You are an expert compiler engineer producing valid Semantic IR v0.5 documents.\n\
         Generate strictly a JSON document matching the Semantic IR v0.5 schema.\n\
         The document must contain 'ir_version': '0.5', 'spec_version': 1, 'wit_contract', and 'functions'.\n\
         Do not include explanations or markdown formatting outside the JSON block."
    }

    /// Builds the user prompt given the component specification, overall system spec, and prior diagnostics.
    #[must_use]
    pub fn build_user_prompt(
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[Diagnostic],
    ) -> String {
        let mut prompt = format!(
            "Generate Semantic IR v0.5 for component '{component_id}'.\n\n\
             System Specification:\n{}\n",
            serde_json::to_string_pretty(spec).unwrap_or_default()
        );

        if !diagnostics.is_empty() {
            prompt.push_str(
                "\nPrevious compilation or verification diagnostics (please fix these issues):\n",
            );
            for diag in diagnostics {
                prompt.push_str(&format!(
                    "- [{}] {}: {}\n",
                    if diag.code.starts_with('E') {
                        "ERROR"
                    } else {
                        "WARNING"
                    },
                    diag.code,
                    diag.message
                ));
            }
        }

        prompt
    }

    fn record_metrics(
        &self,
        duration: std::time::Duration,
        prompt_tokens: Option<u64>,
        completion_tokens: Option<u64>,
        diagnostics_count: usize,
        success: bool,
    ) {
        if let Ok(mut guard) = self.last_metrics.lock() {
            *guard = Some(AgentExecutionMetrics {
                provider_name: self.config.provider.clone(),
                mode: GenerationMode::Real,
                duration,
                prompt_tokens,
                completion_tokens,
                diagnostics_count,
                success,
            });
        }
    }
}

/// Extracts a JSON `Value` from raw model output, stripping markdown code fences if present.
pub fn extract_json_from_response(raw_text: &str) -> Result<Value, AgentError> {
    let trimmed = raw_text.trim();

    // Check for markdown code fences (```json ... ``` or ``` ... ```)
    let candidate = if let Some(start_idx) = trimmed.find("```") {
        let after_fence = &trimmed[start_idx + 3..];
        let content_start = if let Some(stripped) = after_fence.strip_prefix("json") {
            stripped
        } else {
            after_fence
        };
        // Find closing fence
        if let Some(end_idx) = content_start.find("```") {
            content_start[..end_idx].trim()
        } else {
            content_start.trim()
        }
    } else if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start <= end {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    serde_json::from_str(candidate)
        .map_err(|e| AgentError::Json(format!("Failed to parse JSON proposal: {e}")))
}

impl LlmProvider for HttpLlmProvider {
    fn name(&self) -> &str {
        &self.config.provider
    }

    fn mode(&self) -> GenerationMode {
        GenerationMode::Real
    }

    #[allow(clippy::too_many_lines)]
    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[Diagnostic],
    ) -> Result<Value, AgentError> {
        let start = Instant::now();

        if spec.find_component(component_id).is_none() {
            self.record_metrics(start.elapsed(), None, None, diagnostics.len(), false);
            return Err(AgentError::ComponentNotFound(component_id.to_string()));
        }

        // Validate API Key presence before establishing any network connection
        let Some(api_key) = &self.config.api_key else {
            self.record_metrics(start.elapsed(), None, None, diagnostics.len(), false);
            return Err(AgentError::MissingApiKey(self.config.provider.clone()));
        };

        let system_prompt = Self::build_system_prompt();
        let user_prompt = Self::build_user_prompt(component_id, spec, diagnostics);

        let is_openai = self.config.provider.eq_ignore_ascii_case("openai");
        let max_tokens = self.config.max_tokens.unwrap_or(4096);

        let request_payload = if is_openai {
            serde_json::json!({
                "model": &self.config.model,
                "max_tokens": max_tokens,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": user_prompt }
                ]
            })
        } else {
            // Anthropic (default) wire format
            serde_json::json!({
                "model": &self.config.model,
                "max_tokens": max_tokens,
                "system": system_prompt,
                "messages": [
                    { "role": "user", "content": user_prompt }
                ]
            })
        };

        let agent = ureq::AgentBuilder::new()
            .timeout(self.config.timeout)
            .build();

        let mut req = agent.post(&self.config.endpoint);
        req = req.set("content-type", "application/json");

        if is_openai {
            req = req.set("authorization", &format!("Bearer {api_key}"));
        } else {
            req = req
                .set("x-api-key", api_key)
                .set("anthropic-version", "2023-06-01");
        }

        let response = match req.send_json(request_payload) {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                self.record_metrics(start.elapsed(), None, None, diagnostics.len(), false);
                if code == 401 || code == 403 {
                    return Err(AgentError::Http(format!(
                        "Authentication failed (HTTP {code}): verify AGENT_NATIVE_LLM_KEY"
                    )));
                }
                let mut err_body = String::new();
                let _ = resp.into_reader().take(500).read_to_string(&mut err_body);
                let clean_msg = err_body.lines().next().unwrap_or("").trim();
                return Err(AgentError::Http(format!(
                    "LLM endpoint returned HTTP {code}: {clean_msg}"
                )));
            }
            Err(ureq::Error::Transport(transport_err)) => {
                self.record_metrics(start.elapsed(), None, None, diagnostics.len(), false);
                return Err(AgentError::Http(format!(
                    "Transport failure: {transport_err}"
                )));
            }
        };

        // Read response body with upper bound limit
        let mut body_str = String::new();
        if let Err(e) = response
            .into_reader()
            .take(MAX_RESPONSE_BYTES as u64)
            .read_to_string(&mut body_str)
        {
            self.record_metrics(start.elapsed(), None, None, diagnostics.len(), false);
            return Err(AgentError::Http(format!(
                "Failed to read LLM response stream: {e}"
            )));
        }

        let resp_json: Value = match serde_json::from_str(&body_str) {
            Ok(val) => val,
            Err(e) => {
                self.record_metrics(start.elapsed(), None, None, diagnostics.len(), false);
                return Err(AgentError::Json(format!(
                    "Failed to parse LLM response JSON: {e}"
                )));
            }
        };

        let (extracted_text, prompt_tokens, completion_tokens) = if is_openai {
            let text = resp_json
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or_default();
            let p_tokens = resp_json
                .get("usage")
                .and_then(|u| u.get("prompt_tokens"))
                .and_then(Value::as_u64);
            let c_tokens = resp_json
                .get("usage")
                .and_then(|u| u.get("completion_tokens"))
                .and_then(Value::as_u64);
            (text, p_tokens, c_tokens)
        } else {
            // Anthropic format
            let text = resp_json
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| {
                    arr.iter().find_map(|item| {
                        if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                            item.get("text").and_then(|t| t.as_str())
                        } else {
                            None
                        }
                    })
                })
                .or_else(|| resp_json.get("content").and_then(|c| c.as_str()))
                .unwrap_or_default();
            let p_tokens = resp_json
                .get("usage")
                .and_then(|u| u.get("input_tokens"))
                .and_then(Value::as_u64);
            let c_tokens = resp_json
                .get("usage")
                .and_then(|u| u.get("output_tokens"))
                .and_then(Value::as_u64);
            (text, p_tokens, c_tokens)
        };

        let ir_value = match extract_json_from_response(extracted_text) {
            Ok(v) => v,
            Err(err) => {
                self.record_metrics(
                    start.elapsed(),
                    prompt_tokens,
                    completion_tokens,
                    diagnostics.len(),
                    false,
                );
                return Err(err);
            }
        };

        // Validate Semantic IR structure using official verifier
        let verifier_diags = agent_native_ir::verify_ir(&ir_value);
        let error_diags: Vec<_> = verifier_diags
            .iter()
            .filter(|d| d.code.starts_with('E'))
            .collect();

        if !error_diags.is_empty() {
            self.record_metrics(
                start.elapsed(),
                prompt_tokens,
                completion_tokens,
                diagnostics.len(),
                false,
            );
            let formatted_errors = error_diags
                .iter()
                .map(|d| format!("[{}] {} (at {})", d.code, d.message, d.path))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(AgentError::InvalidIr(
                component_id.to_string(),
                format!("Semantic IR v0.5 schema validation failed: {formatted_errors}"),
            ));
        }

        self.record_metrics(
            start.elapsed(),
            prompt_tokens,
            completion_tokens,
            diagnostics.len(),
            true,
        );
        Ok(ir_value)
    }

    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics> {
        self.last_metrics.lock().ok().and_then(|g| g.clone())
    }
}
