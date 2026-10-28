//! Integration tests for LLM providers, offline generation modes, and fallback chaining.
//!
//! Verifies acceptance criteria AC-01 through AC-06 of AN-024.
//! All tests run 100% offline without requiring internet access or API secrets.

use std::time::Duration;

use serde_json::json;
use wasma_cli::agent::{
    AgentError, FallbackLlmProvider, FixtureLlmProvider, GenerationMode, HttpLlmProvider,
    LlmConfig, LlmProvider, SynthesizerLlmProvider,
};
use wasma_cli::spec::RequirementsInterpreter;
use wasma_cli::spec::interpreter::MarkdownRequirementsInterpreter;
use wasma_cli::spec::model::{
    ComponentSpec, EntityField, EntitySpec, OperationParam, OperationSpec, SpecMetadata, SystemSpec,
};

fn create_sample_spec() -> SystemSpec {
    SystemSpec {
        schema: None,
        spec_version: "0.1.0".to_string(),
        metadata: SpecMetadata {
            title: "Math Service".to_string(),
            description: "Sample math service".to_string(),
            source_type: "narrative".to_string(),
            source_files: vec!["spec.md".to_string()],
            last_updated: "2026-10-19T00:00:00Z".to_string(),
        },
        components: vec![ComponentSpec {
            id: "math".to_string(),
            name: "Math Component".to_string(),
            description: "Math operations".to_string(),
            wit_file: "math.wit".to_string(),
            world: "math-world".to_string(),
            export_interfaces: vec!["ops".to_string()],
            import_interfaces: vec![],
            capabilities: vec![],
            dependencies: vec![],
        }],
        requirements: vec![],
        entities: vec![EntitySpec {
            name: "point_sum".to_string(),
            description: "Sum and doubled pair".to_string(),
            fields: vec![
                EntityField {
                    name: "sum".to_string(),
                    field_type: "i32".to_string(),
                    description: "Sum of coords".to_string(),
                },
                EntityField {
                    name: "doubled".to_string(),
                    field_type: "i32".to_string(),
                    description: "Doubled sum".to_string(),
                },
            ],
        }],
        operations: vec![
            OperationSpec {
                name: "double_value".to_string(),
                component: "math".to_string(),
                wit_interface: "ops".to_string(),
                params: vec![OperationParam {
                    name: "x".to_string(),
                    param_type: "i32".to_string(),
                }],
                returns: "i32".to_string(),
                rules_applied: vec![],
                effects: vec![],
            },
            OperationSpec {
                name: "make_point_sum".to_string(),
                component: "math".to_string(),
                wit_interface: "ops".to_string(),
                params: vec![
                    OperationParam {
                        name: "x".to_string(),
                        param_type: "i32".to_string(),
                    },
                    OperationParam {
                        name: "y".to_string(),
                        param_type: "i32".to_string(),
                    },
                ],
                returns: "point_sum".to_string(),
                rules_applied: vec![],
                effects: vec![],
            },
        ],
    }
}

#[test]
fn test_fixture_llm_provider_success_and_miss() {
    let spec = create_sample_spec();
    let mut fixture_provider = FixtureLlmProvider::new("test-fixture");

    assert_eq!(fixture_provider.mode(), GenerationMode::Fixtures);
    assert_eq!(fixture_provider.name(), "test-fixture");

    let golden_ir = json!({
        "ir_version": "0.5",
        "spec_version": 1,
        "wit_contract": {
            "package": "an:math@0.1.0",
            "world": "math-world",
            "export_interface": "ops",
            "import_interfaces": [],
            "wit_file": "math.wit"
        },
        "functions": [
            {
                "name": "double_value",
                "export": true,
                "wit_interface": "ops",
                "params": [{ "name": "x", "type": "i32" }],
                "returns": "i32",
                "requirements": [],
                "body": {
                    "op": "mul",
                    "left": { "op": "param", "name": "x" },
                    "right": { "op": "const", "type": "i32", "value": 2 }
                }
            }
        ]
    });

    // Register golden fixture for component "math"
    fixture_provider.register_fixture("math", golden_ir.clone());

    // Propose IR for existing component
    let proposed = fixture_provider.propose_ir("math", &spec, &[]).unwrap();
    assert_eq!(proposed, golden_ir);

    let metrics = fixture_provider.last_execution_metrics().unwrap();
    assert_eq!(metrics.mode, GenerationMode::Fixtures);
    assert!(metrics.success);

    // Request non-existent component in spec -> ComponentNotFound
    let err = fixture_provider
        .propose_ir("unknown_comp", &spec, &[])
        .unwrap_err();
    assert_eq!(
        err,
        AgentError::ComponentNotFound("unknown_comp".to_string())
    );

    // Create a spec with another component without fixture -> FixtureNotFound
    let mut other_spec = spec.clone();
    other_spec.components.push(ComponentSpec {
        id: "unregistered".to_string(),
        name: "Unregistered".to_string(),
        description: "None".to_string(),
        wit_file: String::new(),
        world: String::new(),
        export_interfaces: vec![],
        import_interfaces: vec![],
        capabilities: vec![],
        dependencies: vec![],
    });

    let err2 = fixture_provider
        .propose_ir("unregistered", &other_spec, &[])
        .unwrap_err();
    assert_eq!(
        err2,
        AgentError::FixtureNotFound("unregistered".to_string())
    );
}

#[test]
fn test_synthesizer_llm_provider_generates_valid_ir() {
    let spec = create_sample_spec();
    let synth = SynthesizerLlmProvider::new("test-synthesizer");

    assert_eq!(synth.mode(), GenerationMode::Synthesizer);

    let ir_value = synth
        .propose_ir("math", &spec, &[])
        .expect("synthesis should succeed");

    assert_eq!(ir_value["ir_version"], "0.5");
    assert_eq!(ir_value["spec_version"], 1);
    assert_eq!(ir_value["wit_contract"]["export_interface"], "ops");

    let fns = ir_value["functions"].as_array().expect("functions array");
    assert_eq!(fns.len(), 2);
    assert_eq!(fns[0]["name"], "double_value");
    assert_eq!(fns[1]["name"], "make_point_sum");

    // Semantic IR v0.5 verifier must confirm zero diagnostic errors
    let diags = wasma_ir::verify_ir(&ir_value);
    let errors: Vec<_> = diags.iter().filter(|d| d.code.starts_with('E')).collect();
    assert!(errors.is_empty(), "Verifier reported errors: {errors:?}");

    let metrics = synth.last_execution_metrics().unwrap();
    assert_eq!(metrics.mode, GenerationMode::Synthesizer);
    assert!(metrics.success);
}

#[test]
fn test_synthesizer_from_markdown_narrative() {
    let narrative = r"# Sistema de Perfil de Usuario

## 1. Visión general del producto
Servicio para gestión de perfiles de usuario.
1. **Administrador de Perfil (`profile-manager`):** Maneja puntuaciones y perfiles.

## 2. Entidades de Dominio
### Puntuación de Usuario (`UserScore`)
- `score_id`: Identificador de la puntuación.
- `subtotal_cents`: Monto neto base.
- `total_cents`: Monto total acumulado.

## 3. Componente A: Perfil (`profile-manager`)
### Reglas
- **`PROF-001` (Cálculo de puntaje):** Asigna puntos base según actividad.

## 4. Contratos de Interfaz (WIT)
### Interfaz Exportada por `profile-manager`
```wit
interface ops {
    calculate-score: func(subtotal-cents: s32) -> s32;
}
```
";

    let interpreter = MarkdownRequirementsInterpreter;
    let spec = interpreter
        .interpret_narrative(narrative, "spec.md")
        .unwrap();
    let synth = SynthesizerLlmProvider::default();

    let ir = synth.propose_ir("profile-manager", &spec, &[]).unwrap();
    assert_eq!(ir["ir_version"], "0.5");

    let diags = wasma_ir::verify_ir(&ir);
    let errors: Vec<_> = diags.iter().filter(|d| d.code.starts_with('E')).collect();
    assert!(errors.is_empty(), "Verifier errors: {errors:?}");
}

#[test]
fn test_http_llm_provider_configuration_and_offline_behavior() {
    let config = LlmConfig {
        provider: "openai".to_string(),
        endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
        model: "gpt-4o".to_string(),
        api_key: None, // Missing key
        timeout: Duration::from_secs(5),
        max_tokens: Some(2048),
        temperature: Some(0),
    };

    let http_provider = HttpLlmProvider::new(config);
    assert_eq!(http_provider.mode(), GenerationMode::Real);

    let spec = create_sample_spec();

    // When API key is missing, fail explicitly with MissingApiKey
    let err = http_provider.propose_ir("math", &spec, &[]).unwrap_err();
    assert_eq!(err, AgentError::MissingApiKey("openai".to_string()));

    // Prompt construction check
    let prompt = HttpLlmProvider::build_user_prompt("math", &spec, &[]);
    assert!(prompt.contains("math"));
    assert!(prompt.contains("System Specification"));
}

#[test]
fn test_fallback_chain_from_http_to_synthesizer() {
    let spec = create_sample_spec();

    // Primary: Http provider with missing key (will fail)
    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: "https://api.anthropic.com/v1/messages".to_string(),
        model: "claude-3-7-sonnet".to_string(),
        api_key: None,
        timeout: Duration::from_secs(10),
        max_tokens: Some(4096),
        temperature: Some(0),
    };
    let primary = Box::new(HttpLlmProvider::new(config));

    // Secondary: Synthesizer provider (will succeed offline)
    let secondary = Box::new(SynthesizerLlmProvider::new("fallback-synth"));

    let fallback = FallbackLlmProvider::new("resilient-agent", vec![primary, secondary]);

    let ir = fallback
        .propose_ir("math", &spec, &[])
        .expect("fallback should succeed via synthesizer");
    assert_eq!(ir["ir_version"], "0.5");

    let metrics = fallback.last_execution_metrics().unwrap();
    assert_eq!(metrics.mode, GenerationMode::Synthesizer);
    assert!(metrics.provider_name.contains("fallback-synth"));
    assert!(metrics.success);
}
