//! Core provider abstractions, execution modes, and metrics for LLM generation.
//!
//! # Risk R-12 Mitigation & Technical Honesty
//!
//! An explicit distinction exists between [`GenerationMode::Real`],
//! [`GenerationMode::Fixtures`], and [`GenerationMode::Synthesizer`].
//!
//! While `Real` invokes an external LLM via HTTP, `Fixtures` and `Synthesizer`
//! are deterministic offline helpers intended strictly for testing, CI regression,
//! and predictable toolchain verification. They must **never** be presented
//! or mistaken as autonomous AI generative capability.

use std::fmt::Display;
use std::time::Duration;

use agent_native_ir::Diagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::error::AgentError;
use crate::spec::model::SystemSpec;

/// Explicit mode under which code/IR generation operates.
///
/// Ensures compliance with Risk R-12 mitigation: no test harness or heuristic
/// rule translator may masquerade as an LLM model invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationMode {
    /// True generative capability via external LLM model.
    Real,
    /// Pre-recorded deterministic fixtures for CI and offline regression.
    Fixtures,
    /// Direct heuristic rule-based translation from specification to IR.
    Synthesizer,
}

impl Display for GenerationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Real => write!(f, "real"),
            Self::Fixtures => write!(f, "fixtures"),
            Self::Synthesizer => write!(f, "synthesizer"),
        }
    }
}

/// Execution telemetry and observability metrics for a generation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentExecutionMetrics {
    /// Provider name that executed or attempted the generation.
    pub provider_name: String,
    /// Generation mode used (explicitly auditable).
    pub mode: GenerationMode,
    /// Wall-clock duration taken for generation.
    pub duration: Duration,
    /// Prompt tokens if reported by provider (None for offline/heuristic).
    pub prompt_tokens: Option<u64>,
    /// Completion tokens if reported by provider (None for offline/heuristic).
    pub completion_tokens: Option<u64>,
    /// Number of diagnostics provided for iterative repair.
    pub diagnostics_count: usize,
    /// Whether the generation was successful.
    pub success: bool,
}

impl AgentExecutionMetrics {
    #[must_use]
    pub fn new_offline(
        provider_name: impl Into<String>,
        mode: GenerationMode,
        duration: Duration,
        success: bool,
    ) -> Self {
        Self {
            provider_name: provider_name.into(),
            mode,
            duration,
            prompt_tokens: None,
            completion_tokens: None,
            diagnostics_count: 0,
            success,
        }
    }
}

/// Configuration settings for LLM providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
    pub max_tokens: Option<u32>,
    pub temperature: Option<u32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            model: "claude-3-7-sonnet-20250219".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            max_tokens: Some(4096),
            temperature: Some(0),
        }
    }
}

impl LlmConfig {
    /// Loads configuration from standard environment variables with fallbacks.
    #[must_use]
    pub fn from_env() -> Self {
        let provider =
            std::env::var("AGENT_NATIVE_LLM_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
        let endpoint = std::env::var("AGENT_NATIVE_LLM_ENDPOINT")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1/messages".to_string());
        let model = std::env::var("AGENT_NATIVE_LLM_MODEL")
            .unwrap_or_else(|_| "claude-3-7-sonnet-20250219".to_string());
        let api_key = std::env::var("AGENT_NATIVE_LLM_KEY").ok();
        let timeout_secs = std::env::var("AGENT_NATIVE_LLM_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30);

        Self {
            provider,
            endpoint,
            model,
            api_key,
            timeout: Duration::from_secs(timeout_secs),
            max_tokens: Some(4096),
            temperature: Some(0),
        }
    }
}

/// Pluggable contract for proposing Semantic IR from a [`SystemSpec`].
pub trait LlmProvider: Send + Sync {
    /// Human-readable identifier for this provider.
    fn name(&self) -> &str;

    /// Explicit operational mode of this provider.
    fn mode(&self) -> GenerationMode;

    /// Proposes a Semantic IR v0.5 JSON document for the specified component.
    ///
    /// If prior verification diagnostics are provided, the provider attempts
    /// self-repair or guided generation fixing the reported issues.
    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[Diagnostic],
    ) -> Result<Value, AgentError>;

    /// Returns the metrics of the most recent execution, if any.
    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics>;
}
