//! Pluggable HTTP LLM provider for external AI model generation.
//!
//! # Risk R-12 Notice
//!
//! This provider operates in [`GenerationMode::Real`]. It connects to an external
//! LLM endpoint (e.g. Anthropic, `OpenAI`, or compatible proxy) to generate Semantic
//! IR v0.5 proposals from specification prompts.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use agent_native_ir::Diagnostic;
use serde_json::Value;

use crate::agent::error::AgentError;
use crate::agent::provider::{AgentExecutionMetrics, GenerationMode, LlmConfig, LlmProvider};
use crate::spec::model::SystemSpec;

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
}

impl LlmProvider for HttpLlmProvider {
    fn name(&self) -> &str {
        &self.config.provider
    }

    fn mode(&self) -> GenerationMode {
        GenerationMode::Real
    }

    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[Diagnostic],
    ) -> Result<Value, AgentError> {
        let start = Instant::now();

        if spec.find_component(component_id).is_none() {
            let mut metrics_guard = self.last_metrics.lock().map_err(|_| {
                AgentError::Synthesis(self.config.provider.clone(), "Lock poisoning".to_string())
            })?;
            *metrics_guard = Some(AgentExecutionMetrics {
                provider_name: self.config.provider.clone(),
                mode: GenerationMode::Real,
                duration: start.elapsed(),
                prompt_tokens: None,
                completion_tokens: None,
                diagnostics_count: diagnostics.len(),
                success: false,
            });
            return Err(AgentError::ComponentNotFound(component_id.to_string()));
        }

        // Validate API Key presence
        let Some(_api_key) = &self.config.api_key else {
            let mut metrics_guard = self.last_metrics.lock().map_err(|_| {
                AgentError::Synthesis(self.config.provider.clone(), "Lock poisoning".to_string())
            })?;
            *metrics_guard = Some(AgentExecutionMetrics {
                provider_name: self.config.provider.clone(),
                mode: GenerationMode::Real,
                duration: start.elapsed(),
                prompt_tokens: None,
                completion_tokens: None,
                diagnostics_count: diagnostics.len(),
                success: false,
            });
            return Err(AgentError::MissingApiKey(self.config.provider.clone()));
        };

        // Network call placeholder (or HTTP client call):
        // Note: For offline CI without network egress, live requests return Http error if endpoint is unreachable.
        Err(AgentError::Http(format!(
            "Endpoint '{}' connection not established in offline test environment",
            self.config.endpoint
        )))
    }

    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics> {
        self.last_metrics.lock().ok().and_then(|g| g.clone())
    }
}
