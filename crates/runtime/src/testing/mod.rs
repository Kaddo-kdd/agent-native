//! YAML-driven acceptance executor for Agent-Native components (AN-019).
//!
//! See `knowledge/delivery/work-items/in-progress/AN-019-executor-yaml.md`
//! and ADR-005 for context. The module stays test-oriented: nothing here is
//! loaded into the production runtime path.
//!
//! # Entry points
//!
//! - [`parse_suite`] — load a `.yaml` file into a typed [`Suite`].
//! - [`run_suite`] — execute every case against a loaded [`Component`],
//!   returning a [`SuiteReport`].
//!
//! Stubs declared in the suite's `host_stubs:` block are deny-by-default:
//! the executor refuses to run a case whose stub targets an import the
//! [`HostPolicy`] does not authorise.
//!
//! [`Component`]: crate::engine::Component
//! [`HostPolicy`]: crate::policy::HostPolicy

pub mod errors;
pub mod executor;
pub mod host_ctx;
pub mod report;
pub mod stub;
pub mod suite;

pub use errors::ExecutorError;
pub use executor::run_suite;
pub use host_ctx::{HostCtx, RecordedCall};
pub use report::{CaseResult, CaseStatus, SuiteReport};
pub use stub::{HostStubRegistry, StubKind};
pub use suite::{Case, Expected, ExpectedHostCall, HostState, Suite, parse_suite};
