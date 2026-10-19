//! Rule-based heuristic translator from `SystemSpec` to Semantic IR v0.5.
//!
//! # Risk R-12 Notice
//!
//! This provider operates in [`GenerationMode::Synthesizer`]. It translates entities,
//! interfaces, and operations structurally into Semantic IR JSON without invoking
//! any AI model or probabilistic generation. It is strictly a deterministic
//! compiler/synthesizer pass and must never be represented as AI code generation.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use agent_native_ir::{verify_ir, Diagnostic};
use serde_json::{json, Value};

use crate::agent::error::AgentError;
use crate::agent::provider::{AgentExecutionMetrics, GenerationMode, LlmProvider};
use crate::spec::model::{OperationSpec, SystemSpec};

/// Heuristic rule-based synthesizer provider.
#[derive(Debug, Clone)]
pub struct SynthesizerLlmProvider {
    name: String,
    last_metrics: Arc<Mutex<Option<AgentExecutionMetrics>>>,
}

impl SynthesizerLlmProvider {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            last_metrics: Arc::new(Mutex::new(None)),
        }
    }

    fn synthesize_function(
        op: &OperationSpec,
        spec: &SystemSpec,
    ) -> Result<Value, AgentError> {
        let params_json: Vec<Value> = op
            .params
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "type": p.param_type
                })
            })
            .collect();

        // Trace requirements associated with this operation or component
        let mut req_ids: Vec<String> = spec
            .requirements
            .iter()
            .filter(|r| r.component == op.component)
            .map(|r| r.id.clone())
            .collect();

        if req_ids.is_empty() {
            req_ids.push(format!("REQ-{}-001", op.component.to_uppercase()));
        }

        // Synthesize a valid default expression body matching the return type
        let body = if op.returns == "i32" {
            if let Some(first_param) = op.params.first() {
                if first_param.param_type == "i32" {
                    json!({
                        "op": "mul",
                        "left": { "op": "param", "name": first_param.name },
                        "right": { "op": "const", "type": "i32", "value": 2 }
                    })
                } else {
                    json!({ "op": "const", "type": "i32", "value": 0 })
                }
            } else {
                json!({ "op": "const", "type": "i32", "value": 0 })
            }
        } else if op.returns == "bool" {
            json!({ "op": "const", "type": "bool", "value": true })
        } else if let Some(entity) = spec.entities.iter().find(|e| e.name == op.returns) {
            // Record construct
            let mut fields_map = serde_json::Map::new();
            for field in &entity.fields {
                let field_expr = if field.field_type == "i32" {
                    if let Some(p) = op.params.iter().find(|p| p.param_type == "i32") {
                        json!({ "op": "param", "name": p.name })
                    } else {
                        json!({ "op": "const", "type": "i32", "value": 0 })
                    }
                } else if field.field_type == "bool" {
                    json!({ "op": "const", "type": "bool", "value": true })
                } else {
                    return Err(AgentError::Synthesis(
                        op.name.clone(),
                        format!("Unsupported field type '{}' for entity '{}'", field.field_type, entity.name),
                    ));
                };
                fields_map.insert(field.name.clone(), field_expr);
            }
            json!({
                "op": "record.construct",
                "type": entity.name,
                "fields": fields_map
            })
        } else {
            return Err(AgentError::Synthesis(
                op.name.clone(),
                format!("Unsupported return type '{}'", op.returns),
            ));
        };

        // Semantic IR function name convention (snake_case)
        let fn_name = op.name.replace('-', "_");

        Ok(json!({
            "name": fn_name,
            "export": true,
            "wit_interface": if op.wit_interface.is_empty() { "ops" } else { &op.wit_interface },
            "params": params_json,
            "returns": op.returns,
            "requirements": req_ids,
            "body": body
        }))
    }
}

impl Default for SynthesizerLlmProvider {
    fn default() -> Self {
        Self::new("heuristic-synthesizer")
    }
}

impl LlmProvider for SynthesizerLlmProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn mode(&self) -> GenerationMode {
        GenerationMode::Synthesizer
    }

    #[allow(clippy::too_many_lines)]
    fn propose_ir(
        &self,
        component_id: &str,
        spec: &SystemSpec,
        diagnostics: &[Diagnostic],
    ) -> Result<Value, AgentError> {
        let start = Instant::now();

        let Some(comp) = spec.find_component(component_id) else {
            let mut metrics_guard = self.last_metrics.lock().map_err(|_| {
                AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string())
            })?;
            *metrics_guard = Some(AgentExecutionMetrics {
                provider_name: self.name.clone(),
                mode: GenerationMode::Synthesizer,
                duration: start.elapsed(),
                prompt_tokens: None,
                completion_tokens: None,
                diagnostics_count: diagnostics.len(),
                success: false,
            });
            return Err(AgentError::ComponentNotFound(component_id.to_string()));
        };

        // Synthesize types (entities)
        let types: Vec<Value> = spec
            .entities
            .iter()
            .map(|e| {
                let fields: Vec<Value> = e
                    .fields
                    .iter()
                    .map(|f| {
                        json!({
                            "name": f.name,
                            "type": f.field_type
                        })
                    })
                    .collect();
                json!({
                    "name": e.name,
                    "fields": fields
                })
            })
            .collect();

        // Synthesize functions from operations belonging to this component
        let comp_ops: Vec<&OperationSpec> = spec
            .operations
            .iter()
            .filter(|o| o.component == component_id)
            .collect();

        let mut functions = Vec::new();
        for op in comp_ops {
            let func_val = Self::synthesize_function(op, spec)?;
            functions.push(func_val);
        }

        if functions.is_empty() {
            // Provide at least one default function if none defined
            functions.push(json!({
                "name": "noop",
                "export": true,
                "wit_interface": "ops",
                "params": [],
                "returns": "i32",
                "requirements": [],
                "body": {
                    "op": "const",
                    "type": "i32",
                    "value": 0
                }
            }));
        }

        let export_iface = comp
            .export_interfaces
            .first()
            .map_or("ops", |s| s.as_str());

        let wit_contract = json!({
            "package": "an:synthesized@0.1.0",
            "world": if comp.world.is_empty() { "service" } else { &comp.world },
            "export_interface": export_iface,
            "import_interfaces": comp.import_interfaces,
            "wit_file": if comp.wit_file.is_empty() { format!("{component_id}.wit") } else { comp.wit_file.clone() }
        });

        let mut ir_doc = json!({
            "ir_version": "0.5",
            "spec_version": 1,
            "wit_contract": wit_contract,
            "functions": functions
        });

        if !types.is_empty() {
            ir_doc["types"] = Value::Array(types);
        }

        // Validate the generated IR using `verify_ir`
        let diags = verify_ir(&ir_doc);
        let error_diags: Vec<_> = diags.iter().filter(|d| d.code.starts_with('E')).collect();
        if !error_diags.is_empty() {
            let error_msgs = error_diags
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ");
            let mut metrics_guard = self.last_metrics.lock().map_err(|_| {
                AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string())
            })?;
            *metrics_guard = Some(AgentExecutionMetrics {
                provider_name: self.name.clone(),
                mode: GenerationMode::Synthesizer,
                duration: start.elapsed(),
                prompt_tokens: None,
                completion_tokens: None,
                diagnostics_count: diagnostics.len(),
                success: false,
            });
            return Err(AgentError::InvalidIr(component_id.to_string(), error_msgs));
        }

        let mut metrics_guard = self.last_metrics.lock().map_err(|_| {
            AgentError::Synthesis(self.name.clone(), "Lock poisoning".to_string())
        })?;
        *metrics_guard = Some(AgentExecutionMetrics {
            provider_name: self.name.clone(),
            mode: GenerationMode::Synthesizer,
            duration: start.elapsed(),
            prompt_tokens: None,
            completion_tokens: None,
            diagnostics_count: diagnostics.len(),
            success: true,
        });

        Ok(ir_doc)
    }

    fn last_execution_metrics(&self) -> Option<AgentExecutionMetrics> {
        self.last_metrics.lock().ok().and_then(|g| g.clone())
    }
}
