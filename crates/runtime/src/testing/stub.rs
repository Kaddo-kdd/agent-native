//! Stub registry + concrete stub kinds (AN-019).
//!
//! Stubs are declared in YAML and resolved to Rust closures that run
//! inside `wasmtime::Linker::func_wrap`. The three supported kinds
//! (`StateReader`, `StateWriter`, `Noop`) have strict `(args) -> ret`
//! signatures; any mismatch is a typed [`ExecutorError::SignatureMismatch`].

use std::collections::HashMap;

use crate::testing::errors::ExecutorError;
use crate::testing::suite::StubKindDecl;

/// Runtime-ready stub kind, resolved from the declarative YAML form.
#[derive(Debug, Clone, Copy)]
pub enum StubKind {
    /// Signature `(i32) -> i32`. Reads `host_state.kv[arg[key_arg]]`.
    StateReader { key_arg: usize },
    /// Signature `(i32, i32) -> i32`. Writes `host_state.kv[arg[key_arg]]
    /// = arg[value_arg]`, returns `arg[value_arg]`.
    StateWriter { key_arg: usize, value_arg: usize },
    /// Signature `() -> i32`. Returns `returns`.
    Noop { returns: i32 },
}

impl StubKind {
    #[must_use]
    pub fn arity(&self) -> usize {
        match self {
            StubKind::StateReader { .. } => 1,
            StubKind::StateWriter { .. } => 2,
            StubKind::Noop { .. } => 0,
        }
    }

    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            StubKind::StateReader { .. } => "state_reader",
            StubKind::StateWriter { .. } => "state_writer",
            StubKind::Noop { .. } => "noop",
        }
    }
}

impl From<StubKindDecl> for StubKind {
    fn from(decl: StubKindDecl) -> Self {
        match decl {
            StubKindDecl::StateReader { key_arg } => StubKind::StateReader { key_arg },
            StubKindDecl::StateWriter { key_arg, value_arg } => {
                StubKind::StateWriter { key_arg, value_arg }
            }
            StubKindDecl::Noop { returns } => StubKind::Noop { returns },
        }
    }
}

/// Registry of stubs by `(interface, operation)`. Populated from the
/// suite's `host_stubs:` section.
#[derive(Debug, Default, Clone)]
pub struct HostStubRegistry {
    stubs: HashMap<(String, String), StubKind>,
}

impl HostStubRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, interface: &str, operation: &str, kind: StubKind) -> &mut Self {
        self.stubs
            .insert((interface.to_string(), operation.to_string()), kind);
        self
    }

    #[must_use]
    pub fn get(&self, interface: &str, operation: &str) -> Option<&StubKind> {
        self.stubs
            .get(&(interface.to_string(), operation.to_string()))
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stubs.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.stubs.len()
    }

    /// Iterates `(key, kind)` entries for the executor's linker builder.
    pub fn iter_entries(&self) -> impl Iterator<Item = (&(String, String), &StubKind)> {
        self.stubs.iter()
    }

    /// Validates a stub's declared kind against the expected arity of
    /// an import observed by the runtime.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError::SignatureMismatch`] when the declared
    /// stub kind cannot satisfy the observed arity.
    pub fn validate_arity(
        &self,
        interface: &str,
        operation: &str,
        observed_arity: usize,
    ) -> Result<(), ExecutorError> {
        let Some(kind) = self.get(interface, operation) else {
            return Err(ExecutorError::StubMissing {
                interface: interface.to_string(),
                operation: operation.to_string(),
            });
        };
        if kind.arity() != observed_arity {
            return Err(ExecutorError::SignatureMismatch {
                interface: interface.to_string(),
                operation: operation.to_string(),
                declared_kind: kind.kind_name().to_string(),
            });
        }
        Ok(())
    }
}
