---
type: work-item
id: AN-049
title: Deterministic WASI 0.2 console I/O stubs and IR v0.6 types in YAML acceptance executor
work_item_type: feature
status: completed
knowledge_level: K3
initiative: INI-007
roadmap_initiative: INIT-07-03
domains:
  - Runtime & Memory
  - Semantic IR & Verification
related_adrs:
  - ADR-004
  - ADR-005
created_at: '2026-10-20'
completed_at: '2026-10-20'
---

# AN-049 — Deterministic WASI 0.2 console I/O stubs and IR v0.6 types in YAML acceptance executor

## Context & Objective

`INIT-07-03` completes `RM-07` (`INI-007`) by extending the declarative YAML acceptance executor (`crates/runtime/src/testing/`) with deterministic in-memory console I/O (`stdin`, `expected_stdout`, `expected_stderr`) and IR v0.6 types (`string` arguments/returns and `result<T, E>` expectations), enabling reproducible console I/O acceptance suites without requiring a real TTY.

## Scope Delivered

1. **YAML Schema Extensions (`crates/runtime/src/testing/suite.rs`):**
   - Added optional `stdin: Option<String>`, `expected_stdout: Option<String>`, and `expected_stderr: Option<String>` fields to `Case`.
   - Extended `ArgValue` with `ArgValue::String(String)`.
   - Extended `Expected` with `Expected::Bool(bool)`, `Expected::String(String)`, and `Expected::Result(ExpectedResult)` (`{ ok: ... }` / `{ err: ... }`).

2. **Deterministic WASI 0.2 Execution in Executor (`crates/runtime/src/testing/executor.rs`):**
   - Delegated WASI 0.2 Component Model cases to `Component::invoke_with_wasi` using `WasiIoConfig::capturing().with_stdin(...)`, preserving per-case `Store` isolation (`ADR-005`) and `HostPolicy` admission checks (`ADR-004`).
   - Compared captured `stdout` and `stderr` bytes deterministically against `expected_stdout` and `expected_stderr`, reporting mismatches as `CaseStatus::Fail` with structured diagnostic evidence.
   - Preserved all 32 existing `AN-019` / `AN-020` YAML executor regression tests.

## Verification Evidence

- `crates/runtime/tests/yaml_wasi_io.rs`: 3/3 tests passing (`yaml_suite_executes_wasi_stdin_stdout_stderr_deterministically_without_tty`, `yaml_suite_reports_stdout_and_stderr_mismatch_as_fail_with_evidence`, `yaml_suite_executes_ir_v06_string_and_result_cases`).
- `crates/runtime/tests/executor_yaml.rs`: 32/32 tests passing.
