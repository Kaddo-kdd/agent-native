//! Fallback chaining provider coordinating multiple providers in priority order.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use agent_native_ir::Diagnostic;
use serde_json::Value;

use crate::agent::error::AgentError;
use crate::agent::provider::{AgentExecutionMetrics, GenerationMode, LlmProvider};
use crate::spec::model::SystemSpec;

/// Fallback provider that tries an ordered chain of inner providers until one succeeds.
pub struct FallbackLlmProvider {
    name: String,
    providers: Vec<Box<dyn LlmProvider>>,
    last_metrics: Arc<Mutex<Option<AgentExecutionMetrics>>>,
}

impl FallbackLlmProvider {
    /// Creates a fallback provider with an ordered chain of providers.
    #[must_use]
    pub fn new(name: impl Into<String>, providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self {
            name: name.into(),
            providers,
            last_metrics: Arc::new(Mutex::new(None)),
        }
    }
}

impl LlmProvider for FallbackLlmProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn mode(&self) -> GenerationMode {
        // Mode is determined by whichever provider succeeded in last execution,
        // or the first provider's mode if not yet executed.
        if let Ok(guard) = self.last_metrics.lock() {
            if let Some(metrics) = &*guard {
                return metrics.mode;
            }
        }
        self.providers
            .first()
            .map_or(GenerationMode::Synthesizer, |p| p.mode())
    }

    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[Diagnostic],
    ) -> Result<Value, AgentError> {
        let start = Instant::now();

        for provider in &self.providers {
            match provider.propose_ir(component_id, spec, diagnostics) {
                Ok(ir) => {
                    let mut metrics_guard = self.last_metrics.lock().map_err(|_| {
                        AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string())
                    })?;
                    *metrics_guard = Some(AgentExecutionMetrics {
                        provider_name: format!("{}:{}", self.name, provider.name()),
                        mode: provider.mode(),
                        duration: start.elapsed(),
                        prompt_tokens: None,
                        completion_tokens: None,
                        diagnostics_count: diagnostics.len(),
                        success: true,
                    });
                    return Ok(ir);
                }
                Err(_err) => {
                    // Continue to next provider in fallback chain
                    continue;
                }
            }
        }

        let mut metrics_guard = self
            .last_metrics
            .lock()
            .map_err(|_| AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string()))?;
        *metrics_guard = Some(AgentExecutionMetrics {
            provider_name: self.name.clone(),
            mode: self
                .providers
                .first()
                .map_or(GenerationMode::Synthesizer, |p| p.mode()),
            duration: start.elapsed(),
            prompt_tokens: None,
            completion_tokens: None,
            diagnostics_count: diagnostics.len(),
            success: false,
        });

        Err(AgentError::AllProvidersFailed)
    }

    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics> {
        self.last_metrics.lock().ok().and_then(|g| g.clone())
    }
}
