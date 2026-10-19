# Agent-Native Examples

This directory and member crates provide canonical examples of how to consume, embed, and deploy `agent-native` components.

## Available Examples

### 1. `embed_runtime` (Canonical Rust Embedding Example)
- **Location:** [`crates/runtime/examples/embed_runtime.rs`](../crates/runtime/examples/embed_runtime.rs)
- **Target:** `cargo run --example embed_runtime`
- **Description:** Demonstrates how an external Rust application embeds `agent-native-runtime`.
  - Initializes `Engine`.
  - Configures strict security policies with `HostPolicy::deny_all()`.
  - Loads WebAssembly Component Model binaries.
  - Invokes WIT-exported operations (`double-value`, `make-point-sum`) passing and unpacking typed `Val` values.
  - Demonstrates typed error handling with `RuntimeError`.
  - Demonstrates content-addressed caching and session isolation using `Resolver::reset()`.

#### Quickstart:
```bash
# Run with the default Component Model fixture:
cargo run --example embed_runtime

# Or run with a custom compiled .wasm binary:
cargo run --example embed_runtime -- path/to/component.wasm
```

---

### 2. `aws-lambda`
- **Location:** [`examples/aws-lambda/`](./aws-lambda/)
- **Description:** Serverless deployment example running an Agent-Native component on AWS Lambda using the custom runtime (`provided.al2023`).

---

### 3. `aws-lambda-benchmark`
- **Location:** [`examples/aws-lambda-benchmark/`](./aws-lambda-benchmark/)
- **Description:** Benchmarking suite comparing cold-start and warm-invocation performance of native Rust binaries vs Agent-Native WASM components.
