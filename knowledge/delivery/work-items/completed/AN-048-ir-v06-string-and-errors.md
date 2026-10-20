---
type: work-item
id: AN-048
title: Semantic IR v0.6 extension for `string`, `result<T, E>` errors, and `--experimental` policy
work_item_type: feature
status: completed
knowledge_level: K4
initiative: INI-007
roadmap_initiative: INIT-07-02
domains:
  - Semantic IR & Verification
  - Compilation
  - Runtime & Memory
related_adrs:
  - ADR-001
  - ADR-002
  - ADR-003
  - ADR-004
created_at: '2026-10-20'
completed_at: '2026-10-20'
---

# AN-048 — Semantic IR v0.6 extension for `string`, `result<T, E>`, and `--experimental` policy

## Context & Objective

`INIT-07-02` extends Semantic IR from the stable `v0.5` baseline to `v0.6` behind an explicit `--experimental` opt-in flag (`VerifyOptions { allow_experimental: true }`, `CompileContext::with_experimental(true)`, and CLI `--experimental` on `verify` and `build`), without breaking or altering `v0.5` semantics or deterministic codegen hashes.

## Scope Delivered

1. **Versioning & Opt-In Policy (`ADR-003` / `R-09`):**
   - `ir_version: "0.5"` remains the default stable baseline across `agent-native-ir`, `agent-native-compiler`, and `agent-native` CLI.
   - `ir_version: "0.6"` is rejected by default with `E_EXPERIMENTAL_IR_VERSION` unless `VerifyOptions { allow_experimental: true }` / `--experimental` is explicitly enabled.
   - `string` and `result<T, E>` types and v0.6 operators are strictly rejected in `v0.1..=v0.5` even when `allow_experimental` is set.
   - Published experimental JSON Schema at `crates/ir/schemas/ir.schema.v0.6.json`.

2. **Type System & Verifier (`crates/ir`):**
   - Extended `IrType` with `IrType::String` (`"string"`) and `IrType::Result { ok, err, repr }` (`"result<T, E>"`).
   - Added verification for:
     - `const` string literals (`{ "op": "const", "type": "string", "value": "..." }`).
     - `eq` over `(string, string) -> bool`.
     - `string.len(value: string) -> i32`.
     - `string.concat(left: string, right: string) -> string`.
     - `result.ok(type: result<T, E>, value: T) -> result<T, E>`.
     - `result.err(type: result<T, E>, value: E) -> result<T, E>`.
     - `result.is_ok(value: result<T, E>) -> bool`.
     - `result.unwrap_ok(value: result<T, E>, default: T) -> T` (total, non-trapping).
     - `result.unwrap_err(value: result<T, E>, default: E) -> E` (total, non-trapping).

3. **Deterministic Canonical ABI Lowering (`crates/compiler`):**
   - Exported `cabi_realloc` in v0.6 modules with 4-byte bump-pointer alignment invariance.
   - Boxed flattened `(ptr, len)` string parameters into 8-byte `(data_ptr, byte_len)` linear-memory descriptors in function prologues, matching the Canonical ABI `retptr` layout.
   - Lowered `string.len`, `string.concat` (`memory.copy`), bytewise string `eq`, and 12-byte Canonical ABI `result<T, E>` (`discriminant` at `+0`, payload at `+4` / `+1` for `result<bool, bool>`).
   - Preserved 100% byte-identical WASM output and SHA-256 digests for `v0.1..=v0.5`.

4. **Runtime & CLI Integration (`crates/runtime`, `crates/cli`):**
   - Extended `component_val_matches_type` in `crates/runtime/src/invoke.rs` for `Val::String`, `Val::Result`, and `Val::Record`.
   - Added `--experimental` flag to `agent-native verify` and `agent-native build`.

## Verification Evidence

- `crates/compiler/tests/ir_v06_experimental.rs`: 3/3 tests passing (`E_EXPERIMENTAL_IR_VERSION` enforcement, v0.5 rejection of `string`/`result`, deterministic v0.6 compilation and `wasmparser` validation).
- `crates/runtime/tests/ir_v06_execution.rs`: 2/2 tests passing (end-to-end execution of `greet`, `byte-len`, `is-admin`, `safe-div`, `describe-div`, `div-or-default`, and `record` with `string` fields).
- `crates/compiler/tests/determinism.rs`: 2/2 tests passing.
