//! Granular WASI Preview 2 authorization policy tests under `Requested ∩ Granted` (AN-045).
//!
//! Covers:
//! - AC-01: Deny-by-default (`HostPolicy::deny_all()`) rejects `wasi:cli/stdout@0.2.0`.
//! - AC-02: Independence of `allow_wasi_stdout()`, `allow_wasi_stderr()`, `allow_wasi_stdin()`,
//!   and pre-instantiate rejection before any guest code (even `start`) executes.
//! - AC-03: `allow_wasi_stdio()` never grants `filesystem`, `sockets`, `clocks`, `random`,
//!   `wasi:cli/environment`, or `wasi:cli/exit`.
//! - AC-04: `ComponentManifest` v0.1 (`wit_interfaces`, no `categories`) parsed from YAML & JSON,
//!   intersected (`Requested ∩ Granted`) per `Tenant` without self-granting.
//! - AC-05: End-to-end scenarios across `Component` and `Tenant`.
//! - AC-06: Auxiliary `wasi:io/*` interfaces (`streams`, `error`, `poll`) granted via
//!   `allow_wasi_stdout()` never grant or link `wasi:cli/stdin`.
//! - AC-07: Exact WIT version matching (`@0.2.0` granted vs `@0.2.3` / `@0.3.0` rejected).

use wasma_runtime::{
    ComponentManifest, Engine, ExecutionLimits, HostPolicy, RuntimeError, Tenant, TenantConfig,
    Val, WASI_CLI_STDERR_0_2_0, WASI_CLI_STDIN_0_2_0, WASI_CLI_STDOUT_0_2_0, WASI_IO_ERROR_0_2_0,
    WASI_IO_POLL_0_2_0, WASI_IO_STREAMS_0_2_0, WasiIoConfig,
};

/// Builds a minimal WASI Preview 2 Component Model binary that imports **only**
/// `wasi:io/{error,poll,streams}@{ver}` and `wasi:cli/stdout@{ver}` (no `stdin`, no `stderr`).
fn build_stdout_only_component(ver: &str) -> Vec<u8> {
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

  (type $io_streams_ty
    (instance
      (alias outer 1 $io_error_res (type $err_outer))
      (export $err "error" (type (eq $err_outer)))
      (type $stream_error_def (variant (case "last-operation-failed" (own $err)) (case "closed")))
      (export $stream_error "stream-error" (type (eq $stream_error_def)))
      (export $output-stream "output-stream" (type (sub resource)))
      (export "[method]output-stream.blocking-write-and-flush" (func (param "self" (borrow $output-stream)) (param "contents" (list u8)) (result (result (error $stream_error)))))
    )
  )
  (import "wasi:io/streams@{ver}" (instance $io_streams (type $io_streams_ty)))
  (alias export $io_streams "output-stream" (type $output_stream))

  (type $stdout_ty
    (instance
      (alias outer 1 $output_stream (type $out_outer))
      (export $out "output-stream" (type (eq $out_outer)))
      (export "get-stdout" (func (result (own $out))))
    )
  )
  (import "wasi:cli/stdout@{ver}" (instance $cli_stdout (type $stdout_ty)))

  (core module $mem_mod
    (memory (export "memory") 1)
    (data (i32.const 1024) "hello-stdout-only")
  )
  (core instance $mem (instantiate $mem_mod))

  (alias export $cli_stdout "get-stdout" (func $get_stdout_f))
  (alias export $io_streams "[method]output-stream.blocking-write-and-flush" (func $blocking_write_f))

  (core func $get_stdout (canon lower (func $get_stdout_f)))
  (core func $blocking_write (canon lower (func $blocking_write_f) (memory $mem "memory")))

  (core module $app
    (import "env" "memory" (memory 1))
    (import "wasi" "get-stdout" (func $get_stdout (result i32)))
    (import "wasi" "blocking-write" (func $blocking_write (param i32 i32 i32 i32)))

    (func (export "greet") (result i32)
      (local $h i32)
      (local.set $h (call $get_stdout))
      (call $blocking_write (local.get $h) (i32.const 1024) (i32.const 17) (i32.const 64))
      (i32.const 17)
    )
  )

  (core instance $app_inst
    (instantiate $app
      (with "env" (instance $mem))
      (with "wasi" (instance
        (export "get-stdout" (func $get_stdout))
        (export "blocking-write" (func $blocking_write))
      ))
    )
  )

  (func $greet_export (result s32) (canon lift (core func $app_inst "greet")))
  (export "greet" (func $greet_export))
)"#
    );
    wat::parse_str(&wat).expect("valid stdout-only WASI component WAT")
}

/// Builds a WASI Preview 2 component that imports `stdout`, `stdin`, and `wasi:io/*` at `{ver}`,
/// and includes a Core WASM `(start ...)` function that traps (`unreachable`) if the component
/// is ever instantiated. Used to prove that policy denial occurs strictly **pre-instantiate**
/// before any guest code can run (AC-02, AC-06).
fn build_stdout_and_stdin_component_with_start_trap(ver: &str) -> Vec<u8> {
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

  (type $io_streams_ty
    (instance
      (alias outer 1 $io_error_res (type $err_outer))
      (export $err "error" (type (eq $err_outer)))
      (type $stream_error_def (variant (case "last-operation-failed" (own $err)) (case "closed")))
      (export $stream_error "stream-error" (type (eq $stream_error_def)))
      (export $input-stream "input-stream" (type (sub resource)))
      (export $output-stream "output-stream" (type (sub resource)))
      (export "[method]input-stream.read" (func (param "self" (borrow $input-stream)) (param "len" u64) (result (result (list u8) (error $stream_error)))))
      (export "[method]output-stream.blocking-write-and-flush" (func (param "self" (borrow $output-stream)) (param "contents" (list u8)) (result (result (error $stream_error)))))
    )
  )
  (import "wasi:io/streams@{ver}" (instance $io_streams (type $io_streams_ty)))
  (alias export $io_streams "input-stream" (type $input_stream))
  (alias export $io_streams "output-stream" (type $output_stream))

  (type $stdout_ty
    (instance
      (alias outer 1 $output_stream (type $out_outer))
      (export $out "output-stream" (type (eq $out_outer)))
      (export "get-stdout" (func (result (own $out))))
    )
  )
  (import "wasi:cli/stdout@{ver}" (instance $cli_stdout (type $stdout_ty)))

  (type $stdin_ty
    (instance
      (alias outer 1 $input_stream (type $in_outer))
      (export $in "input-stream" (type (eq $in_outer)))
      (export "get-stdin" (func (result (own $in))))
    )
  )
  (import "wasi:cli/stdin@{ver}" (instance $cli_stdin (type $stdin_ty)))

  (core module $app
    (func $explode (unreachable))
    (start $explode)
    (func (export "run") (result i32) (i32.const 1))
  )
  (core instance $app_inst (instantiate $app))
  (func $run_export (result s32) (canon lift (core func $app_inst "run")))
  (export "run" (func $run_export))
)"#
    );
    wat::parse_str(&wat).expect("valid stdout+stdin component with start trap")
}

/// Builds a WASI Preview 2 component that imports all three stdio streams (`stdin`, `stdout`,
/// `stderr`) + `wasi:io/*` at `{ver}` and echoes `stdin` to `stdout`.
#[allow(clippy::too_many_lines)]
fn build_full_stdio_echo_component(ver: &str) -> Vec<u8> {
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

  (type $io_streams_ty
    (instance
      (alias outer 1 $io_error_res (type $err_outer))
      (export $err "error" (type (eq $err_outer)))
      (type $stream_error_def (variant (case "last-operation-failed" (own $err)) (case "closed")))
      (export $stream_error "stream-error" (type (eq $stream_error_def)))
      (export $input-stream "input-stream" (type (sub resource)))
      (export $output-stream "output-stream" (type (sub resource)))
      (export "[method]input-stream.read" (func (param "self" (borrow $input-stream)) (param "len" u64) (result (result (list u8) (error $stream_error)))))
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
    (memory (export "memory") 1)
    (global $bump (mut i32) (i32.const 4096))
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
  (alias export $io_streams "[method]input-stream.read" (func $stream_read_f))
  (alias export $io_streams "[method]output-stream.blocking-write-and-flush" (func $blocking_write_f))

  (core func $get_stdin (canon lower (func $get_stdin_f)))
  (core func $get_stdout (canon lower (func $get_stdout_f)))
  (core func $stream_read (canon lower (func $stream_read_f) (memory $mem "memory") (realloc (func $mem "cabi_realloc"))))
  (core func $blocking_write (canon lower (func $blocking_write_f) (memory $mem "memory")))

  (core module $app
    (import "env" "memory" (memory 1))
    (import "wasi" "get-stdin" (func $get_stdin (result i32)))
    (import "wasi" "get-stdout" (func $get_stdout (result i32)))
    (import "wasi" "read" (func $read (param i32 i64 i32)))
    (import "wasi" "blocking-write" (func $blocking_write (param i32 i32 i32 i32)))

    (func (export "echo") (param $max_len i32) (result i32)
      (local $in_h i32)
      (local $out_h i32)
      (local $ptr i32)
      (local $len i32)
      (local.set $in_h (call $get_stdin))
      (call $read (local.get $in_h) (i64.extend_i32_u (local.get $max_len)) (i32.const 256))
      (if (result i32) (i32.ne (i32.load (i32.const 256)) (i32.const 0))
        (then (i32.const 0))
        (else
          (local.set $ptr (i32.load (i32.const 260)))
          (local.set $len (i32.load (i32.const 264)))
          (local.set $out_h (call $get_stdout))
          (call $blocking_write (local.get $out_h) (local.get $ptr) (local.get $len) (i32.const 64))
          (local.get $len)
        )
      )
    )
  )

  (core instance $app_inst
    (instantiate $app
      (with "env" (instance $mem))
      (with "wasi" (instance
        (export "get-stdin" (func $get_stdin))
        (export "get-stdout" (func $get_stdout))
        (export "read" (func $stream_read))
        (export "blocking-write" (func $blocking_write))
      ))
    )
  )

  (func $echo_export (param "max-len" u32) (result s32) (canon lift (core func $app_inst "echo")))
  (export "echo" (func $echo_export))
)"#
    );
    wat::parse_str(&wat).expect("valid full-stdio echo component")
}

/// Builds a component importing an arbitrary single WIT interface with 1 dummy function.
fn build_component_with_single_wit_import(iface: &str) -> Vec<u8> {
    let wat = format!(
        r#"(component
  (type $ty
    (instance
      (export "op" (func (result u32)))
    )
  )
  (import "{iface}" (instance $imp (type $ty)))
  (core module $m
    (func (export "noop") (result i32) (i32.const 0))
  )
  (core instance $mi (instantiate $m))
  (func $noop (result s32) (canon lift (core func $mi "noop")))
  (export "noop" (func $noop))
)"#
    );
    wat::parse_str(&wat).expect("valid single-import component")
}

#[test]
fn ac01_deny_by_default_rejects_wasi_stdout_pre_instantiate() {
    let engine = Engine::new().expect("engine");
    let bytes = build_stdout_only_component("0.2.0");
    let component = engine.load_component(&bytes).expect("load");

    let err = component
        .invoke_with_wasi(
            &HostPolicy::deny_all(),
            &ExecutionLimits::unmetered(),
            "greet",
            &[],
            WasiIoConfig::capturing(),
        )
        .expect_err("deny_all must reject WASI component");

    assert!(
        matches!(err, RuntimeError::UnauthorizedImport { .. }),
        "expected UnauthorizedImport under deny_all, got {err:?}"
    );
}

#[test]
fn ac02_and_ac05_helper_independence_and_inspection() {
    // Verify each helper grants ONLY its own stream + `wasi:io/*@0.2.0`, never the other streams.
    let mut stdout_policy = HostPolicy::deny_all();
    stdout_policy.allow_wasi_stdout();

    assert!(stdout_policy.is_wit_interface_allowed(WASI_CLI_STDOUT_0_2_0));
    assert!(stdout_policy.is_wit_interface_allowed(WASI_IO_STREAMS_0_2_0));
    assert!(stdout_policy.is_wit_interface_allowed(WASI_IO_ERROR_0_2_0));
    assert!(stdout_policy.is_wit_interface_allowed(WASI_IO_POLL_0_2_0));
    assert!(!stdout_policy.is_wit_interface_allowed(WASI_CLI_STDIN_0_2_0));
    assert!(!stdout_policy.is_wit_interface_allowed(WASI_CLI_STDERR_0_2_0));

    let mut stderr_policy = HostPolicy::deny_all();
    stderr_policy.allow_wasi_stderr();
    assert!(stderr_policy.is_wit_interface_allowed(WASI_CLI_STDERR_0_2_0));
    assert!(stderr_policy.is_wit_interface_allowed(WASI_IO_STREAMS_0_2_0));
    assert!(!stderr_policy.is_wit_interface_allowed(WASI_CLI_STDIN_0_2_0));
    assert!(!stderr_policy.is_wit_interface_allowed(WASI_CLI_STDOUT_0_2_0));

    let mut stdin_policy = HostPolicy::deny_all();
    stdin_policy.allow_wasi_stdin();
    assert!(stdin_policy.is_wit_interface_allowed(WASI_CLI_STDIN_0_2_0));
    assert!(stdin_policy.is_wit_interface_allowed(WASI_IO_STREAMS_0_2_0));
    assert!(!stdin_policy.is_wit_interface_allowed(WASI_CLI_STDOUT_0_2_0));
    assert!(!stdin_policy.is_wit_interface_allowed(WASI_CLI_STDERR_0_2_0));

    let mut combined_stdio = HostPolicy::deny_all();
    combined_stdio.allow_wasi_stdio();
    assert!(combined_stdio.is_wit_interface_allowed(WASI_CLI_STDIN_0_2_0));
    assert!(combined_stdio.is_wit_interface_allowed(WASI_CLI_STDOUT_0_2_0));
    assert!(combined_stdio.is_wit_interface_allowed(WASI_CLI_STDERR_0_2_0));
    assert!(combined_stdio.is_wit_interface_allowed(WASI_IO_STREAMS_0_2_0));
}

#[test]
fn ac02_and_ac06_stdout_and_aux_io_granted_still_rejects_stdin_pre_instantiate() {
    // Component has `wasi:cli/stdout@0.2.0` + `wasi:io/{error,poll,streams}@0.2.0` granted via
    // `allow_wasi_stdout()`, AND also imports `wasi:cli/stdin@0.2.0`.
    // Furthermore, the component has a `(start ...)` function that traps (`unreachable`) if
    // instantiation is ever attempted.
    // Expected: rejected pre-instantiate with `UnauthorizedImport { module: "wasi:cli/stdin@0.2.0", .. }`
    // and the guest `start` trap NEVER executes.
    let engine = Engine::new().expect("engine");
    let bytes = build_stdout_and_stdin_component_with_start_trap("0.2.0");
    let component = engine.load_component(&bytes).expect("load");

    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_stdout();

    let err = component
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered().with_fuel(10_000),
            "run",
            &[],
            WasiIoConfig::capturing().with_stdin(b"secret-stdin".to_vec()),
        )
        .expect_err("importing stdin when only stdout+io/* is granted must fail pre-instantiate");

    assert!(
        matches!(
            err,
            RuntimeError::UnauthorizedImport {
                ref module,
                ref name,
            } if module == "wasi:cli/stdin@0.2.0" && (name == "input-stream" || name == "get-stdin")
        ),
        "must fail with UnauthorizedImport for wasi:cli/stdin@0.2.0 before start trap runs, got {err:?}"
    );

    // Complementary AC-06 check: a component that only imports `stdout` + `wasi:io/*` succeeds
    // with `allow_wasi_stdout()`, even if `WasiIoConfig` carries stdin bytes, and produces no
    // stderr output.
    let stdout_only = engine
        .load_component(&build_stdout_only_component("0.2.0"))
        .expect("load stdout-only");
    let ok = stdout_only
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "greet",
            &[],
            WasiIoConfig::capturing().with_stdin(b"unused-stdin".to_vec()),
        )
        .expect("stdout-only component must succeed under allow_wasi_stdout()");
    assert_eq!(ok.report.values, vec![Val::S32(17)]);
    assert_eq!(ok.io.stdout_utf8().unwrap(), "hello-stdout-only");
    assert!(ok.io.stderr.is_empty());
}

#[test]
fn ac03_allow_wasi_stdio_never_grants_filesystem_sockets_clocks_random_env_or_exit() {
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_stdio();

    let forbidden_interfaces = [
        "wasi:filesystem/types@0.2.0",
        "wasi:filesystem/preopens@0.2.0",
        "wasi:sockets/network@0.2.0",
        "wasi:sockets/tcp@0.2.0",
        "wasi:clocks/wall-clock@0.2.0",
        "wasi:clocks/monotonic-clock@0.2.0",
        "wasi:random/random@0.2.0",
        "wasi:cli/environment@0.2.0",
        "wasi:cli/exit@0.2.0",
    ];

    for iface in forbidden_interfaces {
        assert!(
            !policy.is_wit_interface_allowed(iface),
            "allow_wasi_stdio() must never allow {iface}"
        );

        let comp = engine
            .load_component(&build_component_with_single_wit_import(iface))
            .expect("load single-import component");
        let err = comp
            .invoke_with_wasi(
                &policy,
                &ExecutionLimits::unmetered(),
                "noop",
                &[],
                WasiIoConfig::default(),
            )
            .expect_err("forbidden WASI interface must be rejected pre-instantiate");

        assert_eq!(
            err,
            RuntimeError::UnauthorizedImport {
                module: iface.to_string(),
                name: "op".to_string(),
            }
        );
    }
}

#[test]
fn ac04_manifest_v01_intersection_and_no_categories_field() {
    // 1. ComponentManifest v0.1 parses YAML and JSON with WASI `wit_interfaces` and
    //    strictly rejects any `categories` field (`deny_unknown_fields`).
    let yaml_with_categories = r#"
version: "0.1"
capabilities:
  categories:
    - stdio
"#;
    assert!(
        ComponentManifest::from_yaml_str(yaml_with_categories).is_err(),
        "manifest v0.1 must reject unknown `categories` field"
    );

    let requested_yaml = r#"
version: "0.1"
capabilities:
  wit_interfaces:
    - "wasi:io/error@0.2.0"
    - "wasi:io/poll@0.2.0"
    - "wasi:io/streams@0.2.0"
    - "wasi:cli/stdin@0.2.0"
    - "wasi:cli/stdout@0.2.0"
    - "wasi:cli/stderr@0.2.0"
    - "wasi:filesystem/preopens@0.2.0"
"#;
    let requested_from_yaml =
        ComponentManifest::from_yaml_str(requested_yaml).expect("valid v0.1 YAML manifest");

    let requested_json = r#"{
  "version": "0.1",
  "capabilities": {
    "wit_interfaces": [
      "wasi:io/error@0.2.0",
      "wasi:io/poll@0.2.0",
      "wasi:io/streams@0.2.0",
      "wasi:cli/stdin@0.2.0",
      "wasi:cli/stdout@0.2.0",
      "wasi:cli/stderr@0.2.0",
      "wasi:filesystem/preopens@0.2.0"
    ]
  }
}"#;
    let requested_from_json =
        ComponentManifest::from_json_str(requested_json).expect("valid v0.1 JSON manifest");
    assert_eq!(requested_from_yaml, requested_from_json);

    // 2. Host operator grants ONLY `allow_wasi_stdout()`.
    //    Effective = Requested ∩ Granted must permit `stdout` + `wasi:io/*` and deny `stdin`,
    //    `stderr`, and `filesystem/preopens` (an untrusted manifest cannot self-grant).
    let mut host_granted = HostPolicy::deny_all();
    host_granted.allow_wasi_stdout();

    let effective = HostPolicy::effective(&requested_from_yaml, &host_granted);
    assert!(effective.is_wit_interface_allowed(WASI_CLI_STDOUT_0_2_0));
    assert!(effective.is_wit_interface_allowed(WASI_IO_STREAMS_0_2_0));
    assert!(effective.is_wit_interface_allowed(WASI_IO_ERROR_0_2_0));
    assert!(effective.is_wit_interface_allowed(WASI_IO_POLL_0_2_0));
    assert!(!effective.is_wit_interface_allowed(WASI_CLI_STDIN_0_2_0));
    assert!(!effective.is_wit_interface_allowed(WASI_CLI_STDERR_0_2_0));
    assert!(!effective.is_wit_interface_allowed("wasi:filesystem/preopens@0.2.0"));
}

#[test]
fn ac05_tenant_with_stdio_vs_tenant_with_stdout_only() {
    let engine = Engine::new().expect("engine");
    let stdout_comp = engine
        .load_component(&build_stdout_only_component("0.2.0"))
        .expect("load stdout-only component");
    let echo_comp = engine
        .load_component(&build_full_stdio_echo_component("0.2.0"))
        .expect("load full-stdio echo component");

    let mut policy_stdout = HostPolicy::deny_all();
    policy_stdout.allow_wasi_stdout();
    let tenant_stdout = Tenant::new(
        TenantConfig::new("tenant-stdout-only")
            .with_policy(policy_stdout)
            .with_fuel_budget(50_000),
    );

    let mut policy_stdio = HostPolicy::deny_all();
    policy_stdio.allow_wasi_stdio();
    let tenant_stdio = Tenant::new(
        TenantConfig::new("tenant-full-stdio")
            .with_policy(policy_stdio)
            .with_fuel_budget(50_000),
    );

    // Both tenants can run the `stdout`-only component.
    let r1 = tenant_stdout
        .invoke_with_wasi(&stdout_comp, "greet", &[], WasiIoConfig::capturing())
        .expect("tenant_stdout can run stdout-only component");
    assert_eq!(r1.io.stdout_utf8().unwrap(), "hello-stdout-only");

    let r2 = tenant_stdio
        .invoke_with_wasi(&stdout_comp, "greet", &[], WasiIoConfig::capturing())
        .expect("tenant_stdio can run stdout-only component");
    assert_eq!(r2.io.stdout_utf8().unwrap(), "hello-stdout-only");

    // Only `tenant_stdio` can run `echo_comp` (which imports `stdin` + `stdout` + `stderr`).
    let denied = tenant_stdout
        .invoke_with_wasi(
            &echo_comp,
            "echo",
            &[Val::U32(32)],
            WasiIoConfig::capturing().with_stdin(b"ping".to_vec()),
        )
        .expect_err("tenant_stdout must reject component importing stdin/stderr");
    assert!(matches!(denied, RuntimeError::UnauthorizedImport { .. }));

    let echoed = tenant_stdio
        .invoke_with_wasi(
            &echo_comp,
            "echo",
            &[Val::U32(32)],
            WasiIoConfig::capturing().with_stdin(b"ping".to_vec()),
        )
        .expect("tenant_stdio must execute echo_comp");
    assert_eq!(echoed.report.values, vec![Val::S32(4)]);
    assert_eq!(echoed.io.stdout_utf8().unwrap(), "ping");
}

#[test]
fn ac07_helpers_enforce_exact_0_2_0_version_and_reject_other_versions() {
    let engine = Engine::new().expect("engine");
    let mut policy = HostPolicy::deny_all();
    policy.allow_wasi_stdio();

    // Component importing @0.2.3 is rejected by `allow_wasi_stdio()` (which grants @0.2.0).
    let comp_023 = engine
        .load_component(&build_stdout_only_component("0.2.3"))
        .expect("load 0.2.3");
    let err_023 = comp_023
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "greet",
            &[],
            WasiIoConfig::capturing(),
        )
        .expect_err("allow_wasi_stdio() (@0.2.0) must reject @0.2.3 imports");
    assert!(
        matches!(
            err_023,
            RuntimeError::UnauthorizedImport { ref module, .. } if module.ends_with("@0.2.3")
        ),
        "expected UnauthorizedImport for @0.2.3, got {err_023:?}"
    );

    // Component importing @0.3.0 is also rejected pre-instantiate.
    let comp_030 = engine
        .load_component(&build_stdout_only_component("0.3.0"))
        .expect("load 0.3.0");
    let err_030 = comp_030
        .invoke_with_wasi(
            &policy,
            &ExecutionLimits::unmetered(),
            "greet",
            &[],
            WasiIoConfig::capturing(),
        )
        .expect_err("allow_wasi_stdio() (@0.2.0) must reject @0.3.0 imports");
    assert!(
        matches!(
            err_030,
            RuntimeError::UnauthorizedImport { ref module, .. } if module.ends_with("@0.3.0")
        ),
        "expected UnauthorizedImport for @0.3.0, got {err_030:?}"
    );
}
