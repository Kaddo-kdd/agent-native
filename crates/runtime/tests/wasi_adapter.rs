//! Integration and spike tests for the WASI Preview 2 in-memory adapter (AN-044).
//!
//! Covers:
//! - AC-01..AC-07 (deterministic in-memory stdin/stdout/stderr, isolation across invocations,
//!   per-interface linker restriction, exact/accumulated/atomic `IoLimitExceeded`, fuel
//!   exhaustion on unbounded write loop).
//! - Spike AC-S1..AC-S4 (exact WIT version matching vs patch mismatch vs incompatible version,
//!   per-interface registration in `wasmtime-wasi 29.0.1`, non-textual `IoOverflow` propagation,
//!   non-blocking `check-write` / `blocking-write-and-flush`).

use agent_native_runtime::{
    Engine, ExecutionLimits, HostPolicy, IoStream, RuntimeError, Tenant, TenantConfig, Val,
    WasiIoConfig,
};

/// Builds a minimal WASI Preview 2 Component Model binary (WAT) parameterized by the `@version`
/// suffix used on its `wasi:io/*` and `wasi:cli/*` imports.
///
/// Exported functions:
/// - `write-stdout(n: u32) -> s32`: writes `n` bytes (`'A'`) to `stdout` in a single `write` call.
/// - `write-stdout-twice(n1: u32, n2: u32) -> s32`: writes `n1` bytes then `n2` bytes to `stdout`.
/// - `blocking-write-stdout(n: u32) -> s32`: writes `n` bytes via `blocking-write-and-flush`.
/// - `write-stderr(n: u32) -> s32`: writes `n` bytes (`'E'`) to `stderr` in a single `write` call.
/// - `echo-stdin-to-stdout(max_len: u32) -> s32`: reads up to `max_len` bytes from `stdin`,
///   writes them to `stdout`, and returns the number of bytes read (`0` on immediate EOF).
/// - `infinite-stdout-loop() -> s32`: loops forever writing 1 byte to `stdout` per iteration.
#[allow(clippy::too_many_lines)]
fn build_wasi_io_component(ver: &str) -> Vec<u8> {
    let wat = format!(
        r#"(component
  (type $io_err_ty
    (instance
      (export "error" (type (sub resource)))
    )
  )
  (import "wasi:io/error@{ver}" (instance $io_error (type $io_err_ty)))
  (alias export $io_error "error" (type $io_error_res))

  (type $io_poll_ty
    (instance
      (export "pollable" (type (sub resource)))
    )
  )
  (import "wasi:io/poll@{ver}" (instance $io_poll (type $io_poll_ty)))
  (alias export $io_poll "pollable" (type $pollable_res))

  (type $io_streams_ty
    (instance
      (alias outer 1 $io_error_res (type $err_outer))
      (export $err "error" (type (eq $err_outer)))
      (type $stream_error_def (variant (case "last-operation-failed" (own $err)) (case "closed")))
      (export $stream_error "stream-error" (type (eq $stream_error_def)))
      (export $input-stream "input-stream" (type (sub resource)))
      (export $output-stream "output-stream" (type (sub resource)))
      (export "[method]input-stream.read" (func (param "self" (borrow $input-stream)) (param "len" u64) (result (result (list u8) (error $stream_error)))))
      (export "[method]output-stream.check-write" (func (param "self" (borrow $output-stream)) (result (result u64 (error $stream_error)))))
      (export "[method]output-stream.write" (func (param "self" (borrow $output-stream)) (param "contents" (list u8)) (result (result (error $stream_error)))))
      (export "[method]output-stream.blocking-write-and-flush" (func (param "self" (borrow $output-stream)) (param "contents" (list u8)) (result (result (error $stream_error)))))
    )
  )
  (import "wasi:io/streams@{ver}" (instance $io_streams (type $io_streams_ty)))
  (alias export $io_streams "input-stream" (type $input_stream))
  (alias export $io_streams "output-stream" (type $output_stream))

  (type $stdin_ty
    (instance
      (alias outer 1 $input_stream (type $in_outer))
      (export $in "input-stream" (type (eq $in_outer)))
      (export "get-stdin" (func (result (own $in))))
    )
  )
  (import "wasi:cli/stdin@{ver}" (instance $cli_stdin (type $stdin_ty)))

  (type $stdout_ty
    (instance
      (alias outer 1 $output_stream (type $out_outer))
      (export $out "output-stream" (type (eq $out_outer)))
      (export "get-stdout" (func (result (own $out))))
    )
  )
  (import "wasi:cli/stdout@{ver}" (instance $cli_stdout (type $stdout_ty)))

  (type $stderr_ty
    (instance
      (alias outer 1 $output_stream (type $out_outer))
      (export $out "output-stream" (type (eq $out_outer)))
      (export "get-stderr" (func (result (own $out))))
    )
  )
  (import "wasi:cli/stderr@{ver}" (instance $cli_stderr (type $stderr_ty)))

  (core module $mem_mod
    (memory (export "memory") 2)
    (global $bump (mut i32) (i32.const 65536))
    (func (export "cabi_realloc") (param $old_ptr i32) (param $old_size i32) (param $align i32) (param $new_size i32) (result i32)
      (local $ret i32)
      (local.set $ret (global.get $bump))
      (global.set $bump (i32.add (global.get $bump) (local.get $new_size)))
      (local.get $ret)
    )
  )
  (core instance $mem (instantiate $mem_mod))

  (alias export $cli_stdin "get-stdin" (func $get_stdin_f))
  (alias export $cli_stdout "get-stdout" (func $get_stdout_f))
  (alias export $cli_stderr "get-stderr" (func $get_stderr_f))
  (alias export $io_streams "[method]input-stream.read" (func $stream_read_f))
  (alias export $io_streams "[method]output-stream.check-write" (func $stream_check_write_f))
  (alias export $io_streams "[method]output-stream.write" (func $stream_write_f))
  (alias export $io_streams "[method]output-stream.blocking-write-and-flush" (func $stream_blocking_write_f))

  (core func $get_stdin (canon lower (func $get_stdin_f)))
  (core func $get_stdout (canon lower (func $get_stdout_f)))
  (core func $get_stderr (canon lower (func $get_stderr_f)))
  (core func $stream_read (canon lower (func $stream_read_f) (memory $mem "memory") (realloc (func $mem "cabi_realloc"))))
  (core func $stream_check_write (canon lower (func $stream_check_write_f) (memory $mem "memory")))
  (core func $stream_write (canon lower (func $stream_write_f) (memory $mem "memory")))
  (core func $stream_blocking_write (canon lower (func $stream_blocking_write_f) (memory $mem "memory")))

  (core module $app
    (import "env" "memory" (memory 2))
    (import "wasi" "get-stdin" (func $get_stdin (result i32)))
    (import "wasi" "get-stdout" (func $get_stdout (result i32)))
    (import "wasi" "get-stderr" (func $get_stderr (result i32)))
    (import "wasi" "read" (func $read (param i32 i64 i32)))
    (import "wasi" "check-write" (func $check_write (param i32 i32)))
    (import "wasi" "write" (func $write (param i32 i32 i32 i32)))
    (import "wasi" "blocking-write" (func $blocking_write (param i32 i32 i32 i32)))

    ;; Fill memory[offset .. offset + len] with `byte`.
    (func $fill (param $offset i32) (param $len i32) (param $byte i32)
      (local $i i32)
      (local.set $i (i32.const 0))
      (block $done
        (loop $lp
          (br_if $done (i32.ge_u (local.get $i) (local.get $len)))
          (i32.store8 (i32.add (local.get $offset) (local.get $i)) (local.get $byte))
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          (br $lp)
        )
      )
    )

    (func (export "write-stdout") (param $n i32) (result i32)
      (local $h i32)
      (call $fill (i32.const 1024) (local.get $n) (i32.const 65))
      (local.set $h (call $get_stdout))
      (call $check_write (local.get $h) (i32.const 0))
      (call $write (local.get $h) (i32.const 1024) (local.get $n) (i32.const 64))
      (local.get $n)
    )

    (func (export "write-stdout-twice") (param $n1 i32) (param $n2 i32) (result i32)
      (local $h i32)
      (call $fill (i32.const 1024) (local.get $n1) (i32.const 65))
      (call $fill (i32.const 8192) (local.get $n2) (i32.const 66))
      (local.set $h (call $get_stdout))
      (call $check_write (local.get $h) (i32.const 0))
      (call $write (local.get $h) (i32.const 1024) (local.get $n1) (i32.const 64))
      (call $check_write (local.get $h) (i32.const 0))
      (call $write (local.get $h) (i32.const 8192) (local.get $n2) (i32.const 64))
      (i32.add (local.get $n1) (local.get $n2))
    )

    (func (export "blocking-write-stdout") (param $n i32) (result i32)
      (local $h i32)
      (local $rem i32)
      (local $chunk i32)
      (call $fill (i32.const 1024) (local.get $n) (i32.const 67))
      (local.set $h (call $get_stdout))
      (local.set $rem (local.get $n))
      (block $done
        (loop $lp
          (br_if $done (i32.le_u (local.get $rem) (i32.const 0)))
          (local.set $chunk
            (select (i32.const 4096) (local.get $rem) (i32.gt_u (local.get $rem) (i32.const 4096)))
          )
          (call $blocking_write (local.get $h) (i32.const 1024) (local.get $chunk) (i32.const 64))
          (local.set $rem (i32.sub (local.get $rem) (local.get $chunk)))
          (br $lp)
        )
      )
      (local.get $n)
    )

    (func (export "write-stderr") (param $n i32) (result i32)
      (local $h i32)
      (call $fill (i32.const 1024) (local.get $n) (i32.const 69))
      (local.set $h (call $get_stderr))
      (call $check_write (local.get $h) (i32.const 0))
      (call $write (local.get $h) (i32.const 1024) (local.get $n) (i32.const 64))
      (local.get $n)
    )

    (func (export "echo-stdin-to-stdout") (param $max_len i32) (result i32)
      (local $in_h i32)
      (local $out_h i32)
      (local $ptr i32)
      (local $len i32)
      (local.set $in_h (call $get_stdin))
      ;; read result at offset 256: [discriminant: i32, ptr_or_err: i32, len: i32]
      (call $read (local.get $in_h) (i64.extend_i32_u (local.get $max_len)) (i32.const 256))
      (if (result i32) (i32.ne (i32.load (i32.const 256)) (i32.const 0))
        (then
          ;; StreamError::Closed or error -> return 0 bytes read
          (i32.const 0)
        )
        (else
          (local.set $ptr (i32.load (i32.const 260)))
          (local.set $len (i32.load (i32.const 264)))
          (local.set $out_h (call $get_stdout))
          (call $blocking_write (local.get $out_h) (local.get $ptr) (local.get $len) (i32.const 64))
          (local.get $len)
        )
      )
    )

    (func (export "infinite-stdout-loop") (result i32)
      (local $h i32)
      (i32.store8 (i32.const 1024) (i32.const 88))
      (local.set $h (call $get_stdout))
      (loop $forever
        (call $write (local.get $h) (i32.const 1024) (i32.const 1) (i32.const 64))
        (br $forever)
      )
      (i32.const 0)
    )
  )

  (core instance $app_inst
    (instantiate $app
      (with "env" (instance $mem))
      (with "wasi" (instance
        (export "get-stdin" (func $get_stdin))
        (export "get-stdout" (func $get_stdout))
        (export "get-stderr" (func $get_stderr))
        (export "read" (func $stream_read))
        (export "check-write" (func $stream_check_write))
        (export "write" (func $stream_write))
        (export "blocking-write" (func $stream_blocking_write))
      ))
    )
  )

  (func $write_stdout_export (param "n" u32) (result s32)
    (canon lift (core func $app_inst "write-stdout"))
  )
  (func $write_stdout_twice_export (param "n1" u32) (param "n2" u32) (result s32)
    (canon lift (core func $app_inst "write-stdout-twice"))
  )
  (func $blocking_write_stdout_export (param "n" u32) (result s32)
    (canon lift (core func $app_inst "blocking-write-stdout"))
  )
  (func $write_stderr_export (param "n" u32) (result s32)
    (canon lift (core func $app_inst "write-stderr"))
  )
  (func $echo_stdin_export (param "max-len" u32) (result s32)
    (canon lift (core func $app_inst "echo-stdin-to-stdout"))
  )
  (func $infinite_stdout_loop_export (result s32)
    (canon lift (core func $app_inst "infinite-stdout-loop"))
  )

  (export "write-stdout" (func $write_stdout_export))
  (export "write-stdout-twice" (func $write_stdout_twice_export))
  (export "blocking-write-stdout" (func $blocking_write_stdout_export))
  (export "write-stderr" (func $write_stderr_export))
  (export "echo-stdin-to-stdout" (func $echo_stdin_export))
  (export "infinite-stdout-loop" (func $infinite_stdout_loop_export))
)"#
    );
    wat::parse_str(&wat).expect("valid WASI component WAT")
}

/// Builds a minimal Component that imports an unsupported WASI interface (`wasi:filesystem/preopens@0.2.3`)
/// alongside `wasi:cli/stdout@0.2.3`.
fn build_component_importing_filesystem() -> Vec<u8> {
    let wat = r#"(component
  (type $fs_ty
    (instance
      (export "get-directories" (func (result (list u32))))
    )
  )
  (import "wasi:filesystem/preopens@0.2.3" (instance $fs (type $fs_ty)))
  (core module $m
    (func (export "noop") (result i32) (i32.const 1))
  )
  (core instance $mi (instantiate $m))
  (func $noop (result s32) (canon lift (core func $mi "noop")))
  (export "noop" (func $noop))
)"#;
    wat::parse_str(wat).expect("valid component WAT with filesystem import")
}

fn policy_for_version(ver: &str) -> HostPolicy {
    let mut policy = HostPolicy::deny_all();
    for iface in [
        format!("wasi:io/error@{ver}"),
        format!("wasi:io/poll@{ver}"),
        format!("wasi:io/streams@{ver}"),
        format!("wasi:cli/stdin@{ver}"),
        format!("wasi:cli/stdout@{ver}"),
        format!("wasi:cli/stderr@{ver}"),
    ] {
        policy.allow_wit_interface(iface);
    }
    policy
}

#[test]
fn wasi_stdout_stderr_and_stdin_deterministic_in_memory() {
    let engine = Engine::new().expect("engine");
    let bytes = build_wasi_io_component("0.2.3");
    let policy = policy_for_version("0.2.3");
    let component = engine
        .load_component_with_policy(&bytes, &policy)
        .expect("load component");

    // 1. Write 5 bytes ('A') to stdout and capture.
    let res = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(5)],
            WasiIoConfig::capturing(),
        )
        .expect("stdout invocation must succeed");
    assert_eq!(res.report.values, vec![Val::S32(5)]);
    assert_eq!(res.io.stdout_utf8().unwrap(), "AAAAA");
    assert!(res.io.stderr.is_empty());

    // 2. Write 4 bytes ('E') to stderr and capture.
    let res_err = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stderr",
            &[Val::U32(4)],
            WasiIoConfig::capturing(),
        )
        .expect("stderr invocation must succeed");
    assert_eq!(res_err.report.values, vec![Val::S32(4)]);
    assert!(res_err.io.stdout.is_empty());
    assert_eq!(res_err.io.stderr_utf8().unwrap(), "EEEE");

    // 3. Echo injected stdin ("hello-wasi") to stdout.
    let res_echo = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "echo-stdin-to-stdout",
            &[Val::U32(64)],
            WasiIoConfig::capturing().with_stdin(b"hello-wasi".to_vec()),
        )
        .expect("stdin->stdout echo must succeed");
    assert_eq!(res_echo.report.values, vec![Val::S32(10)]);
    assert_eq!(res_echo.io.stdout_utf8().unwrap(), "hello-wasi");

    // 4. With stdin = None, reading returns immediate EOF (0 bytes) without blocking.
    let res_eof = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "echo-stdin-to-stdout",
            &[Val::U32(64)],
            WasiIoConfig::capturing(),
        )
        .expect("stdin=None must return immediate EOF");
    assert_eq!(res_eof.report.values, vec![Val::S32(0)]);
    assert!(res_eof.io.stdout.is_empty());
}

#[test]
fn consecutive_invocations_have_isolated_buffers_and_fresh_contexts() {
    let engine = Engine::new().expect("engine");
    let bytes = build_wasi_io_component("0.2.3");
    let policy = policy_for_version("0.2.3");
    let component = engine.load_component(&bytes).expect("load");

    let first = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(6)],
            WasiIoConfig::capturing(),
        )
        .expect("first run");
    assert_eq!(first.io.stdout_utf8().unwrap(), "AAAAAA");

    let second = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(2)],
            WasiIoConfig::capturing(),
        )
        .expect("second run");
    assert_eq!(
        second.io.stdout_utf8().unwrap(),
        "AA",
        "second invocation must never observe residue from first invocation"
    );
}

#[test]
fn blocking_write_and_flush_succeeds_across_multiple_permits() {
    // Spike AC-S4: `check-write` returns a non-zero permit (4096) and `blocking-write-and-flush`
    // cleanly writes payloads larger than a single permit (e.g. 10,000 bytes) without hanging.
    let engine = Engine::new().expect("engine");
    let bytes = build_wasi_io_component("0.2.3");
    let policy = policy_for_version("0.2.3");
    let component = engine.load_component(&bytes).expect("load");

    let res = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "blocking-write-stdout",
            &[Val::U32(10_000)],
            WasiIoConfig::capturing().with_max_output_bytes(16_384, 16_384),
        )
        .expect("10KB blocking-write-and-flush must succeed");
    assert_eq!(res.io.stdout.len(), 10_000);
    assert!(res.io.stdout.iter().all(|&b| b == b'C'));
}

#[test]
fn io_limit_exact_boundary_plus_one_accumulated_and_isolation() {
    // AC-05 & Spike AC-S3:
    // 1. Exact limit succeeds.
    // 2. Limit + 1 fails with typed `RuntimeError::IoLimitExceeded` where `attempted_bytes`
    //    is the accumulated size that would have been retained.
    // 3. Multiple writes crossing the limit report the accumulated `attempted_bytes`.
    // 4. Subsequent invocation starts with fresh counters at 0.
    // 5. Stderr enforces its own independent limit and reports `IoStream::Stderr`.
    let engine = Engine::new().expect("engine");
    let bytes = build_wasi_io_component("0.2.3");
    let policy = policy_for_version("0.2.3");
    let component = engine.load_component(&bytes).expect("load");

    let io_cfg = WasiIoConfig::capturing().with_max_output_bytes(10, 8);

    // (1) Exact boundary: 10 bytes on limit 10 -> OK.
    let exact = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(10)],
            io_cfg.clone(),
        )
        .expect("exact limit must succeed");
    assert_eq!(exact.io.stdout.len(), 10);

    // (2) Limit + 1 in a single write: 11 bytes on limit 10 -> IoLimitExceeded { Stdout, 10, 11 }.
    let err_plus_one = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(11)],
            io_cfg.clone(),
        )
        .expect_err("limit + 1 must fail");
    assert_eq!(
        err_plus_one,
        RuntimeError::IoLimitExceeded {
            stream: IoStream::Stdout,
            limit: 10,
            attempted_bytes: 11,
        }
    );

    // (3) Accumulated writes: 6 bytes (accepted) + 5 bytes (rejected) on limit 10
    //     -> IoLimitExceeded { Stdout, limit: 10, attempted_bytes: 11 }.
    let err_accum = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout-twice",
            &[Val::U32(6), Val::U32(5)],
            io_cfg.clone(),
        )
        .expect_err("accumulated write 6 + 5 > 10 must fail");
    assert_eq!(
        err_accum,
        RuntimeError::IoLimitExceeded {
            stream: IoStream::Stdout,
            limit: 10,
            attempted_bytes: 11,
        }
    );

    // (4) Isolation after overflow: next invocation with 10 bytes succeeds cleanly.
    let after_overflow = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(10)],
            io_cfg.clone(),
        )
        .expect("subsequent invocation must start with clean counter");
    assert_eq!(after_overflow.io.stdout.len(), 10);

    // (5) Stderr limit: 9 bytes on stderr limit 8 -> IoLimitExceeded { Stderr, 8, 9 }.
    let err_stderr = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "write-stderr",
            &[Val::U32(9)],
            io_cfg,
        )
        .expect_err("stderr limit + 1 must fail");
    assert_eq!(
        err_stderr,
        RuntimeError::IoLimitExceeded {
            stream: IoStream::Stderr,
            limit: 8,
            attempted_bytes: 9,
        }
    );
}

#[test]
fn unbounded_stdout_loop_is_stopped_by_fuel_budget() {
    // AC-06: an infinite guest write loop exhausts its fuel budget deterministically.
    let engine = Engine::new().expect("engine");
    let bytes = build_wasi_io_component("0.2.3");
    let policy = policy_for_version("0.2.3");
    let component = engine.load_component(&bytes).expect("load");

    // Disable capture so the fuel limit is hit before any output buffer cap.
    let err = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered().with_fuel(5_000),
            "infinite-stdout-loop",
            &[],
            WasiIoConfig::default(),
        )
        .expect_err("infinite write loop must exhaust fuel");
    assert_eq!(err, RuntimeError::FuelExhausted { limit: 5_000 });
}

#[test]
fn linker_does_not_register_ungranted_or_unsupported_wasi_interfaces() {
    // AC-04:
    // (a) Without policy grant, importing `wasi:filesystem/preopens@0.2.3` is rejected
    //     pre-instantiate with `UnauthorizedImport`.
    // (b) Even if `HostPolicy` grants `wasi:filesystem/preopens@0.2.3`, the WASI linker
    //     only registers the supported console/io interfaces, never `add_to_linker_sync`,
    //     so instantiation fails explicitly (`InstantiationFailed`).
    let engine = Engine::new().expect("engine");
    let bytes = build_component_importing_filesystem();
    let component = engine.load_component(&bytes).expect("load");

    let denied = component
        .invoke_with_wasi(
            &HostPolicy::deny_all(),
            &ExecutionLimits::unmetered(),
            "noop",
            &[],
            WasiIoConfig::default(),
        )
        .expect_err("ungranted filesystem import must fail precheck");
    assert!(
        matches!(
            denied,
            RuntimeError::UnauthorizedImport { ref module, .. }
                if module == "wasi:filesystem/preopens@0.2.3"
        ),
        "expected UnauthorizedImport for wasi:filesystem/preopens@0.2.3, got {denied:?}"
    );

    let mut policy_granting_fs = HostPolicy::deny_all();
    policy_granting_fs.allow_wit_interface("wasi:filesystem/preopens@0.2.3");
    let not_linked = component
        .invoke_with_wasi(
            &policy_granting_fs,
            &ExecutionLimits::unmetered(),
            "noop",
            &[],
            WasiIoConfig::default(),
        )
        .expect_err("filesystem is not linked by the console adapter");
    assert!(
        matches!(not_linked, RuntimeError::InstantiationFailed(_)),
        "expected InstantiationFailed when filesystem is not in linker, got {not_linked:?}"
    );
}

#[test]
fn spike_wit_version_matching_exact_vs_patch_difference_vs_incompatible() {
    // Spike AC-S1:
    // - Wasmtime's linker can resolve `@0.2.0` imports against `@0.2.3` host bindings when
    //   the exact `@0.2.0` identifier is authorised in `HostPolicy`.
    // - Authorising `@0.2.0` when the component imports `@0.2.3` (or vice versa) is rejected
    //   pre-instantiate by `HostPolicy` with `UnauthorizedImport`.
    // - Importing an incompatible major/minor version `@0.3.0` when `@0.2.3` is authorised is
    //   rejected pre-instantiate with `UnauthorizedImport`.
    let engine = Engine::new().expect("engine");
    let comp_020 = engine
        .load_component(&build_wasi_io_component("0.2.0"))
        .expect("load 0.2.0 component");
    let comp_023 = engine
        .load_component(&build_wasi_io_component("0.2.3"))
        .expect("load 0.2.3 component");
    let comp_030 = engine
        .load_component(&build_wasi_io_component("0.3.0"))
        .expect("load 0.3.0 component");

    let policy_020 = policy_for_version("0.2.0");
    let policy_023 = policy_for_version("0.2.3");

    // (a) Exact match @0.2.0 granted and @0.2.0 imported -> succeeds!
    let ok_020 = comp_020
        .invoke_with_wasi(
            &policy_020,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(3)],
            WasiIoConfig::capturing(),
        )
        .expect("exact @0.2.0 policy + @0.2.0 component must succeed");
    assert_eq!(ok_020.io.stdout_utf8().unwrap(), "AAA");

    // (b1) Host grants @0.2.0, component imports @0.2.3 -> rejected pre-instantiate!
    let mismatch_020_vs_023 = comp_023
        .invoke_with_wasi(
            &policy_020,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(3)],
            WasiIoConfig::capturing(),
        )
        .expect_err("granting @0.2.0 must not authorise @0.2.3 import");
    assert!(
        matches!(
            mismatch_020_vs_023,
            RuntimeError::UnauthorizedImport { ref module, .. } if module.ends_with("@0.2.3")
        ),
        "expected UnauthorizedImport for @0.2.3, got {mismatch_020_vs_023:?}"
    );

    // (b2) Host grants @0.2.3, component imports @0.2.0 -> rejected pre-instantiate!
    let mismatch_023_vs_020 = comp_020
        .invoke_with_wasi(
            &policy_023,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(3)],
            WasiIoConfig::capturing(),
        )
        .expect_err("granting @0.2.3 must not authorise @0.2.0 import");
    assert!(
        matches!(
            mismatch_023_vs_020,
            RuntimeError::UnauthorizedImport { ref module, .. } if module.ends_with("@0.2.0")
        ),
        "expected UnauthorizedImport for @0.2.0, got {mismatch_023_vs_020:?}"
    );

    // (c) Host grants @0.2.3, component imports @0.3.0 -> rejected pre-instantiate!
    let incompatible_030 = comp_030
        .invoke_with_wasi(
            &policy_023,
            &ExecutionLimits::unmetered(),
            "write-stdout",
            &[Val::U32(3)],
            WasiIoConfig::capturing(),
        )
        .expect_err("granting @0.2.3 must not authorise @0.3.0 import");
    assert!(
        matches!(
            incompatible_030,
            RuntimeError::UnauthorizedImport { ref module, .. } if module.ends_with("@0.3.0")
        ),
        "expected UnauthorizedImport for @0.3.0, got {incompatible_030:?}"
    );

    // Verify Tenant::invoke_with_wasi also enforces the exact policy and fuel accounting.
    let tenant = Tenant::new(
        TenantConfig::new("tenant-wasi")
            .with_policy(policy_023)
            .with_fuel_budget(50_000),
    );
    let tenant_res = tenant
        .invoke_with_wasi(
            &comp_023,
            "write-stdout",
            &[Val::U32(4)],
            WasiIoConfig::capturing(),
        )
        .expect("tenant invoke_with_wasi must succeed");
    assert_eq!(tenant_res.io.stdout_utf8().unwrap(), "AAAA");
    assert!(tenant_res.report.fuel_consumed.unwrap() > 0);
}
