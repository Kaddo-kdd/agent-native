//! YAML suite data model + parser (AN-019).
//!
//! The format extends the POC `tests/*.yaml` schema with a new top-level
//! `host_stubs:` key. All POC fields keep their original names and
//! semantics (`id`, `requirement`, `function`, `args`, `host_state`,
//! `expected`, `expected_host_calls`) per OQ-02.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::testing::errors::ExecutorError;

/// A whole YAML suite as declared in a `tests/suites/*.yaml` file.
#[derive(Debug, Deserialize, Clone)]
pub struct Suite {
    pub suite: String,
    pub spec_version: u32,
    #[serde(default)]
    pub host_stubs: Vec<StubDecl>,
    pub cases: Vec<Case>,
}

/// Declarative stub binding `(interface, operation) -> StubKind`.
#[derive(Debug, Deserialize, Clone)]
pub struct StubDecl {
    pub interface: String,
    pub operation: String,
    #[serde(flatten)]
    pub kind: StubKindDecl,
}

/// Declarative stub kind (serde-friendly surface for `stub::StubKind`).
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StubKindDecl {
    /// `(i32) -> i32`: read `host_state.kv[arg[key_arg]]`.
    StateReader { key_arg: usize },
    /// `(i32, i32) -> i32`: write `host_state.kv[arg[key_arg]] = arg[value_arg]`,
    /// return `arg[value_arg]`.
    StateWriter { key_arg: usize, value_arg: usize },
    /// `() -> i32`: return the fixed `returns` value.
    Noop { returns: i32 },
}

/// A single case inside a suite.
#[derive(Debug, Deserialize, Clone)]
pub struct Case {
    pub id: String,
    #[serde(default)]
    pub requirement: Option<String>,
    pub function: String,
    #[serde(default)]
    pub args: Vec<i32>,
    #[serde(default)]
    pub host_state: HostState,
    pub expected: Expected,
    #[serde(default)]
    pub expected_host_calls: Option<Vec<ExpectedHostCall>>,
}

/// Initial host state for a case. Only `kv` is modelled in MVP; richer
/// state types are deferred to RM-07 per AN-019 OQ-06.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct HostState {
    #[serde(default)]
    pub kv: BTreeMap<i32, i32>,
}

/// What the case expects the component to return.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Expected {
    Primitive(i32),
    Record(BTreeMap<String, i32>),
}

/// A single expected host call recorded by the executor.
#[derive(Debug, Deserialize, Clone)]
pub struct ExpectedHostCall {
    pub function: String,
    pub args: Vec<i32>,
}

/// Parses a YAML text into a `Suite`. Any schema or syntax error becomes
/// [`ExecutorError::InvalidYaml`].
///
/// # Errors
///
/// Returns [`ExecutorError::InvalidYaml`] if deserialisation fails.
pub fn parse_suite(yaml: &str) -> Result<Suite, ExecutorError> {
    serde_yaml2::from_str(yaml).map_err(|e| ExecutorError::InvalidYaml(e.to_string()))
}
