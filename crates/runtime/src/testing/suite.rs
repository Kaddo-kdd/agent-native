//! YAML suite data model + parser (AN-019 + AN-020 schema).
//!
//! The schema extends the POC `tests/*.yaml` format with:
//!
//! - a new top-level `host_stubs:` key (AN-019),
//! - `args` entries that can be `i32` or `bool` (AN-020 H4b),
//! - `expected_host_calls[*].import` as the canonical field, with
//!   `function` accepted as a temporary alias (AN-020 H4a / OQ-01).
//!
//! Explicit AN-020 contract decisions:
//!
//! - `suite:` and `spec_version:` are **mandatory**. Suites that omit
//!   them (POC narrative-style) are rejected in parse.
//! - `host_state` only supports the `kv:` sub-map. Any other key
//!   (`credit_limits`, `tenant_status`, ...) is rejected in parse.
//! - Short vs qualified `import` names are both accepted; the
//!   normalisation rule is documented in
//!   `knowledge/tech/poc-yaml-contract.md`.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::testing::errors::ExecutorError;

/// A whole YAML suite as declared in a `tests/suites/*.yaml` file.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    #[serde(default)]
    pub requirement: Option<String>,
    pub function: String,
    #[serde(default)]
    pub args: Vec<ArgValue>,
    #[serde(default)]
    pub host_state: HostState,
    pub expected: Expected,
    #[serde(default)]
    pub expected_host_calls: Option<Vec<ExpectedHostCall>>,
}

/// A single argument value in a case. AN-020 adds `bool`; richer types
/// are deferred to INIT-07-02.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum ArgValue {
    Bool(bool),
    I32(i32),
}

/// Initial host state for a case. **AN-020**: only `kv:` is permitted;
/// any other key is rejected in parse (see `poc-yaml-contract.md`).
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
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
///
/// AN-020 (OQ-01): `import` is canonical; `function` is a temporary
/// alias. Declaring both at once is rejected in parse.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExpectedHostCallRaw {
    #[serde(default)]
    pub import: Option<String>,
    #[serde(default)]
    pub function: Option<String>,
    pub args: Vec<i32>,
}

/// Validated expected host call: carries a single canonical name.
#[derive(Debug, Clone)]
pub struct ExpectedHostCall {
    pub import: String,
    pub args: Vec<i32>,
}

impl<'de> Deserialize<'de> for ExpectedHostCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ExpectedHostCallRaw::deserialize(deserializer)?;
        match (raw.import, raw.function) {
            (Some(_), Some(_)) => Err(serde::de::Error::custom(
                "both `import` and `function` set on expected_host_calls entry; declare only one",
            )),
            (Some(name), None) | (None, Some(name)) => {
                validate_import_name(&name).map_err(serde::de::Error::custom)?;
                Ok(ExpectedHostCall {
                    import: name,
                    args: raw.args,
                })
            }
            (None, None) => Err(serde::de::Error::custom(
                "missing `import` field on expected_host_calls entry",
            )),
        }
    }
}

/// Enforces the short-vs-qualified normalisation rule from H4 / OQ-01:
///
/// - A `name` without `.` is a short operation form (`kv_get`).
/// - A `name` with `.` is a fully-qualified `interface.operation` form,
///   with both sides non-empty (no `.op`, no `host.`).
pub(crate) fn validate_import_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("empty `import` name".to_string());
    }
    if let Some((iface, op)) = name.split_once('.') {
        if iface.is_empty() || op.is_empty() {
            return Err(format!(
                "ambiguous `import` name '{name}': both interface and operation must be non-empty"
            ));
        }
    }
    Ok(())
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
