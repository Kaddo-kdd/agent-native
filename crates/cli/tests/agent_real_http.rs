//! Integration tests for [`HttpLlmProvider`] and `--mode real` using in-process mock HTTP servers.
//!
//! # Invariants Verified
//! - 100% offline: tests bind to ephemeral loopback (`127.0.0.1:0`).
//! - No credential leakage in error messages or logs.
//! - Anthropic and `OpenAI` wire protocol compliance.
//! - Extraction of JSON with and without markdown code fences.
//! - Semantic IR schema verification (`verify_ir`) rejection of non-conforming IR proposals.
//! - Accurate token and duration telemetry recording.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use agent_native::agent::{AgentError, HttpLlmProvider, LlmConfig, LlmProvider};
use agent_native::spec::model::{ComponentSpec, SpecMetadata, SystemSpec};

fn cli_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_agent-native"))
}

fn create_sample_spec() -> SystemSpec {
    SystemSpec {
        schema: None,
        spec_version: "0.1.0".to_string(),
        metadata: SpecMetadata {
            title: "Calc Service".to_string(),
            description: "Sample calc service".to_string(),
            source_type: "narrative".to_string(),
            source_files: vec!["spec.md".to_string()],
            last_updated: "2026-10-20T00:00:00Z".to_string(),
        },
        components: vec![ComponentSpec {
            id: "calc".to_string(),
            name: "Calculator".to_string(),
            description: "Calculator component".to_string(),
            wit_file: "calc.wit".to_string(),
            world: "calc-world".to_string(),
            export_interfaces: vec!["ops".to_string()],
            import_interfaces: vec![],
            capabilities: vec![],
            dependencies: vec![],
        }],
        requirements: vec![],
        entities: vec![],
        operations: vec![],
    }
}

const VALID_IR_JSON: &str = r#"{
  "ir_version": "0.5",
  "spec_version": 1,
  "wit_contract": {
    "package": "an:calc@0.1.0",
    "world": "calc-world",
    "export_interface": "ops",
    "import_interfaces": [],
    "wit_file": "calc.wit"
  },
  "functions": [
    {
      "name": "add",
      "export": true,
      "wit_interface": "ops",
      "params": [
        {"name": "a", "type": "i32"},
        {"name": "b", "type": "i32"}
      ],
      "returns": "i32",
      "requirements": ["CALC-001"],
      "body": {
        "op": "add",
        "left": {"op": "param", "name": "a"},
        "right": {"op": "param", "name": "b"}
      }
    }
  ]
}"#;

struct MockServer {
    url: String,
    received: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    addr: SocketAddr,
}

impl Drop for MockServer {
    fn drop(&mut self) {
        // Connect a dummy stream to ensure accept() never blocks test exit
        let _ = TcpStream::connect_timeout(&self.addr, Duration::from_millis(50));
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn spawn_mock_server(status_line: &'static str, response_body: String) -> MockServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind mock listener");
    let addr = listener.local_addr().expect("local addr");
    let url = format!("http://127.0.0.1:{}/v1/messages", addr.port());

    let received_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&received_flag);

    let handle = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));

        // Read headers
        let mut content_length = 0usize;
        {
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
                return;
            }

            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                    break;
                }
                if line.to_ascii_lowercase().starts_with("content-length:") {
                    if let Some(val) = line.split(':').nth(1) {
                        content_length = val.trim().parse().unwrap_or(0);
                    }
                }
            }

            if content_length > 0 {
                let mut body_buf = vec![0u8; content_length];
                let _ = reader.read_exact(&mut body_buf);
            }
        }

        flag_clone.store(true, Ordering::SeqCst);

        let resp = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
        let _ = stream.flush();
    });

    MockServer {
        url,
        received: received_flag,
        handle: Some(handle),
        addr,
    }
}

#[test]
fn test_missing_api_key_fails_early_without_network() {
    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: "http://127.0.0.1:9999/unreachable".to_string(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: None,
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let res = provider.propose_ir("calc", &spec, &[]);
    assert_eq!(
        res.unwrap_err(),
        AgentError::MissingApiKey("anthropic".to_string())
    );

    let metrics = provider.last_execution_metrics().expect("metrics recorded");
    assert!(!metrics.success);
}

#[test]
fn test_anthropic_protocol_success_and_metrics() {
    let response_body = serde_json::json!({
        "id": "msg_01",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": VALID_IR_JSON
            }
        ],
        "usage": {
            "input_tokens": 125,
            "output_tokens": 340
        }
    })
    .to_string();

    let server = spawn_mock_server("200 OK", response_body);

    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: server.url.clone(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: Some("sk-ant-test-key-secret".to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let val = provider
        .propose_ir("calc", &spec, &[])
        .expect("IR proposal ok");

    assert!(server.received.load(Ordering::SeqCst));
    assert_eq!(val["ir_version"], "0.5");
    assert_eq!(val["wit_contract"]["export_interface"], "ops");

    let metrics = provider.last_execution_metrics().expect("metrics");
    assert!(metrics.success);
    assert_eq!(metrics.prompt_tokens, Some(125));
    assert_eq!(metrics.completion_tokens, Some(340));
}

#[test]
fn test_openai_protocol_success_and_metrics() {
    let response_body = serde_json::json!({
        "id": "chatcmpl_01",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": VALID_IR_JSON
                }
            }
        ],
        "usage": {
            "prompt_tokens": 95,
            "completion_tokens": 280
        }
    })
    .to_string();

    let server = spawn_mock_server("200 OK", response_body);

    let config = LlmConfig {
        provider: "openai".to_string(),
        endpoint: server.url.clone(),
        model: "gpt-4o".to_string(),
        api_key: Some("sk-openai-test-key".to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let val = provider
        .propose_ir("calc", &spec, &[])
        .expect("IR proposal ok");

    assert!(server.received.load(Ordering::SeqCst));
    assert_eq!(val["ir_version"], "0.5");

    let metrics = provider.last_execution_metrics().expect("metrics");
    assert!(metrics.success);
    assert_eq!(metrics.prompt_tokens, Some(95));
    assert_eq!(metrics.completion_tokens, Some(280));
}

#[test]
fn test_markdown_fence_extraction() {
    let wrapped_content =
        format!("Here is the generated IR:\n```json\n{VALID_IR_JSON}\n```\nHope this helps!");
    let response_body = serde_json::json!({
        "id": "msg_02",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": wrapped_content
            }
        ],
        "usage": {
            "input_tokens": 100,
            "output_tokens": 200
        }
    })
    .to_string();

    let server = spawn_mock_server("200 OK", response_body);

    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: server.url.clone(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: Some("test-key".to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let val = provider
        .propose_ir("calc", &spec, &[])
        .expect("Parsed fenced JSON");

    assert_eq!(val["ir_version"], "0.5");
}

#[test]
fn test_http_401_unauthorized_error_no_key_leak() {
    let secret_key = "very_sensitive_anthropic_api_key_secret_12345";
    let server = spawn_mock_server(
        "401 Unauthorized",
        "{\"error\":\"invalid_api_key\"}".to_string(),
    );

    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: server.url.clone(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: Some(secret_key.to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let err = provider.propose_ir("calc", &spec, &[]).unwrap_err();

    let err_str = err.to_string();
    assert!(err_str.contains("Authentication failed (HTTP 401)"));
    assert!(
        !err_str.contains(secret_key),
        "API key leaked in error string!"
    );
}

#[test]
fn test_http_500_error_handling() {
    let server = spawn_mock_server(
        "500 Internal Server Error",
        "Internal error details".to_string(),
    );

    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: server.url.clone(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: Some("test-key".to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let err = provider.propose_ir("calc", &spec, &[]).unwrap_err();

    assert!(err.to_string().contains("HTTP 500"));
}

#[test]
fn test_malformed_non_json_response() {
    let server = spawn_mock_server("200 OK", "<html>Not JSON</html>".to_string());

    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: server.url.clone(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: Some("test-key".to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let err = provider.propose_ir("calc", &spec, &[]).unwrap_err();

    assert!(matches!(err, AgentError::Json(_)));
}

#[test]
fn test_invalid_semantic_ir_schema_rejected_by_verify_ir() {
    // Valid JSON but missing required Semantic IR fields (e.g. wit_contract)
    let invalid_ir = r#"{
      "ir_version": "0.5",
      "spec_version": 1,
      "functions": []
    }"#;

    let response_body = serde_json::json!({
        "id": "msg_03",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": invalid_ir
            }
        ]
    })
    .to_string();

    let server = spawn_mock_server("200 OK", response_body);

    let config = LlmConfig {
        provider: "anthropic".to_string(),
        endpoint: server.url.clone(),
        model: "claude-3-7-sonnet-20250219".to_string(),
        api_key: Some("test-key".to_string()),
        timeout: Duration::from_secs(5),
        max_tokens: Some(4096),
        temperature: Some(0),
    };

    let provider = HttpLlmProvider::new(config);
    let spec = create_sample_spec();

    let err = provider.propose_ir("calc", &spec, &[]).unwrap_err();

    assert!(matches!(err, AgentError::InvalidIr(_, _)));
    assert!(
        err.to_string()
            .contains("Semantic IR v0.5 schema validation failed")
    );
}

#[test]
fn test_build_mode_real_cli_e2e_with_mock_server() {
    let response_body = serde_json::json!({
        "id": "msg_e2e",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": VALID_IR_JSON
            }
        ],
        "usage": { "input_tokens": 100, "output_tokens": 200 }
    })
    .to_string();

    let server = spawn_mock_server("200 OK", response_body);

    let temp_dir =
        std::env::temp_dir().join(format!("agent_native_test_real_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let spec_path = temp_dir.join("spec.md");
    let wit_path = temp_dir.join("calc.wit");
    let out_wasm = temp_dir.join("calc.wasm");

    let wit_content = r"package an:calc@0.1.0;

interface ops {
    add: func(a: s32, b: s32) -> s32;
}

world calc-world {
    export ops;
}
";
    std::fs::write(&wit_path, wit_content).expect("write wit");

    let spec_content = r"# Sistema de Calculadora

## 1. Visión general del producto
Servicio para operaciones de cálculo.
1. **Componente A: Calculadora (`calc`):** Maneja adición.

## 2. Entidades de Dominio
### Total (`Total`)
- `value`: Monto acumulado.

## 3. Componente A: Calculadora (`calc`)
### Reglas
- **`CALC-001` (Suma):** Suma dos valores.

## 4. Contratos de Interfaz (WIT)
### Interfaz Exportada por `calc`
```wit
interface ops {
    add: func(a: s32, b: s32) -> s32;
}
```
";
    std::fs::write(&spec_path, spec_content).expect("write spec");

    let bin = cli_binary_path();
    let status = Command::new(&bin)
        .current_dir(&temp_dir)
        .arg("build")
        .arg(&spec_path)
        .arg("--output")
        .arg(&out_wasm)
        .arg("--component")
        .arg("calc")
        .arg("--mode")
        .arg("real")
        .env("AGENT_NATIVE_LLM_PROVIDER", "anthropic")
        .env("AGENT_NATIVE_LLM_ENDPOINT", &server.url)
        .env("AGENT_NATIVE_LLM_KEY", "test-live-key")
        .status()
        .expect("failed to execute cli binary");

    assert!(status.success(), "Process exited with failure: {status:?}");
    assert!(out_wasm.exists());
    let wasm_bytes = std::fs::read(&out_wasm).expect("read wasm");
    assert!(!wasm_bytes.is_empty());

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}
