//! Deterministic pre-recorded fixtures provider for offline CI and test suites.
//!
//! # Risk R-12 Notice
//!
//! This provider operates in [`GenerationMode::Fixtures`]. It plays back pre-verified
//! or golden IR documents without invoking external models. It is strictly a
//! test harness utility and must never be represented as AI code generation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde_json::Value;
use wasma_ir::Diagnostic;

use crate::agent::error::AgentError;
use crate::agent::provider::{AgentExecutionMetrics, GenerationMode, LlmProvider};
use crate::spec::model::SystemSpec;

/// Fixture-based provider mapping `(component_id) -> Semantic IR Value`.
#[derive(Debug, Clone)]
pub struct FixtureLlmProvider {
    name: String,
    fixtures: Arc<HashMap<String, Value>>,
    last_metrics: Arc<Mutex<Option<AgentExecutionMetrics>>>,
}

impl FixtureLlmProvider {
    /// Creates a new fixture provider with an empty catalog.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fixtures: Arc::new(HashMap::new()),
            last_metrics: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a provider from an existing map of fixtures.
    #[must_use]
    pub fn from_map(name: impl Into<String>, fixtures: HashMap<String, Value>) -> Self {
        Self {
            name: name.into(),
            fixtures: Arc::new(fixtures),
            last_metrics: Arc::new(Mutex::new(None)),
        }
    }

    /// Registers a component's pre-recorded Semantic IR fixture.
    pub fn register_fixture(&mut self, component_id: impl Into<String>, ir_json: Value) {
        if let Some(map) = Arc::get_mut(&mut self.fixtures) {
            map.insert(component_id.into(), ir_json);
        } else {
            let mut map = (*self.fixtures).clone();
            map.insert(component_id.into(), ir_json);
            self.fixtures = Arc::new(map);
        }
    }
}

impl LlmProvider for FixtureLlmProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn mode(&self) -> GenerationMode {
        GenerationMode::Fixtures
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
                AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string())
            })?;
            *metrics_guard = Some(AgentExecutionMetrics {
                provider_name: self.name.clone(),
                mode: GenerationMode::Fixtures,
                duration: start.elapsed(),
                prompt_tokens: None,
                completion_tokens: None,
                diagnostics_count: diagnostics.len(),
                success: false,
            });
            return Err(AgentError::ComponentNotFound(component_id.to_string()));
        }

        let result = match self.fixtures.get(component_id) {
            Some(val) => Ok(val.clone()),
            None => Err(AgentError::FixtureNotFound(component_id.to_string())),
        };

        let success = result.is_ok();
        let mut metrics_guard = self
            .last_metrics
            .lock()
            .map_err(|_| AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string()))?;
        *metrics_guard = Some(AgentExecutionMetrics {
            provider_name: self.name.clone(),
            mode: GenerationMode::Fixtures,
            duration: start.elapsed(),
            prompt_tokens: None,
            completion_tokens: None,
            diagnostics_count: diagnostics.len(),
            success,
        });

        result
    }

    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics> {
        self.last_metrics.lock().ok().and_then(|g| g.clone())
    }
}
