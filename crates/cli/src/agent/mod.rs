//! Agent module for LLM-driven synthesis and offline generation modes.
//!
//! # Architecture & ADR-001 Invariants
//!
//! In accordance with ADR-001, agent and orchestration adapters reside
//! within the `agent_native` CLI library (`crates/cli/src/agent/`) as internal
//! modules until the workspace reaches extraction criteria.
//!
//! # Risk R-12 Mitigation: Transparent Generation Modes
//!
//! An explicit distinction exists between [`GenerationMode::Real`],
//! [`GenerationMode::Fixtures`], and [`GenerationMode::Synthesizer`].
//!
//! - `Real`: Live external LLM calls via HTTP.
//! - `Fixtures`: Deterministic replay of pre-recorded IR fixtures for zero-flakiness CI.
//! - `Synthesizer`: Direct heuristic rule translation from `SystemSpec` to Semantic IR v0.5.
//!
//! Offline modes must never masquerade as AI generative capabilities.

pub mod error;
pub mod fallback;
pub mod fixtures;
pub mod provider;
pub mod real;
pub mod synthesizer;

pub use error::AgentError;
pub use fallback::FallbackLlmProvider;
pub use fixtures::FixtureLlmProvider;
pub use provider::{AgentExecutionMetrics, GenerationMode, LlmConfig, LlmProvider};
pub use real::HttpLlmProvider;
pub use synthesizer::SynthesizerLlmProvider;
