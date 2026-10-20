//! Individual invocation latencies for AN-054. Criterion samples are not used.
//!
//! Inputs and WASI buffers are prepared before the timer; results are consumed
//! and dropped after it. Every invoke retains the runtime's fresh-Store policy.

#[path = "latency/fixtures.rs"]
mod fixtures;

use agent_native_runtime::{Engine, ExecutionLimits, Val, WasiIoConfig};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Sampling {
    n: usize,
    warmup: usize,
    calibration: usize,
}

#[derive(Debug, Serialize)]
struct Statistics {
    min: u64,
    p50: u64,
    p95: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    p99: Option<u64>,
    max: u64,
}

fn nearest_rank(sorted: &[u64], percentile: usize) -> u64 {
    assert!(!sorted.is_empty() && (1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}

fn statistics(samples: &[u64]) -> Statistics {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    Statistics {
        min: sorted[0],
        p50: nearest_rank(&sorted, 50),
        p95: nearest_rank(&sorted, 95),
        p99: (sorted.len() >= 1_000).then(|| nearest_rank(&sorted, 99)),
        max: sorted[sorted.len() - 1],
    }
}

fn elapsed_ns(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).expect("elapsed time fits u64")
}

fn measure<P, T, E: Debug>(
    sampling: Sampling,
    mut prepare: impl FnMut() -> P,
    mut invoke: impl FnMut(P) -> Result<T, E>,
) -> Vec<u64> {
    for _ in 0..sampling.warmup {
        let input = prepare();
        black_box(invoke(input).expect("warmup invocation"));
    }
    let mut samples = Vec::with_capacity(sampling.n);
    for _ in 0..sampling.n {
        let input = prepare();
        let start = Instant::now();
        let result = invoke(input);
        let elapsed = elapsed_ns(start);
        black_box(result.expect("measured invocation"));
        samples.push(elapsed);
    }
    samples
}

fn calibration(n: usize) -> Value {
    // Minimum positive observed step is an empirical upper bound on intrinsic
    // clock resolution: it also includes the cost of consecutive Instant reads.
    let mut resolution = None;
    let mut previous = Instant::now();
    for _ in 0..n {
        let now = Instant::now();
        let step = u64::try_from(now.duration_since(previous).as_nanos()).expect("clock step");
        if step > 0 {
            resolution = Some(resolution.map_or(step, |old: u64| old.min(step)));
        }
        previous = now;
    }
    let samples = measure(
        Sampling {
            n,
            warmup: 1_000,
            calibration: n,
        },
        || (),
        |()| {
            black_box(());
            Ok::<_, std::convert::Infallible>(())
        },
    );
    json!({
        "n": n, "warmup": 1_000, "resolution_ns": resolution,
        "resolution_method": "minimum positive step between consecutive Instant reads; upper bound, not intrinsic hardware resolution",
        "overhead_method": "same timer envelope around an empty black_box operation; empirical lower bound on measurement overhead",
        "statistics": statistics(&samples), "samples_ns": samples,
    })
}

fn row(
    name: &str,
    wasm: &[u8],
    operation: &str,
    api: &str,
    arguments: &Value,
    sampling: Sampling,
    samples: &[u64],
) -> Value {
    json!({
        "benchmark": name, "method": "percentil_por_invocacion", "unit": "ns",
        "operation": operation, "api": api, "arguments": arguments,
        "fixture_sha256": format!("{:x}", Sha256::digest(wasm)),
        "n": sampling.n, "warmup": sampling.warmup,
        "statistics": statistics(samples), "samples_ns": samples,
        "fresh_store_per_invocation": true,
        "timed_scope": if name.starts_with("cache_hit") { "Resolver::load(cache hit) + invoke; prepared inputs and result drop outside timer" }
                       else { "invoke only; prepared inputs and result drop outside timer" },
    })
}

// Keep the complete scenario inventory and its timing boundaries together.
#[allow(clippy::too_many_lines)]
fn collect(sampling: Sampling) -> Value {
    // Compilation and loading are outside every measured invocation.
    let identity = fixtures::identity_wasm();
    let point = fixtures::point_sum_wasm();
    let wasi = fixtures::wasi_echo_wasm();
    let v06 = fixtures::v06_wasm();
    let engine = Engine::new().expect("engine");
    let core = engine.load_component(&identity).expect("identity load");
    let cm = engine.load_component(&point).expect("point load");
    let advanced = engine.load_component(&v06).expect("v06 load");
    let policy = fixtures::stdio_policy();
    let echo = engine
        .load_component_with_policy(&wasi, &policy)
        .expect("WASI load");
    let unmetered = ExecutionLimits::unmetered();
    let metered = unmetered.with_fuel(1_000_000);
    let core_args = [Val::S32(42)];
    let point_args = [Val::S32(3), Val::S32(4)];
    let greet_args = [Val::String("world".to_string())];
    let div_args = [Val::S32(42), Val::S32(6)];
    let echo_args = [Val::S32(64), Val::S32(0)];
    let mut resolver = agent_native_runtime::Resolver::new().expect("resolver");
    resolver.load(&identity).expect("prime core");
    resolver.load(&point).expect("prime point");

    let before = calibration(sampling.calibration);
    let mut rows = Vec::new();
    macro_rules! record {
        ($name:expr, $wasm:expr, $operation:expr, $api:expr, $args:expr, $prepare:expr, $invoke:expr) => {
            rows.push(row(
                $name,
                $wasm,
                $operation,
                $api,
                &$args,
                sampling,
                &measure(sampling, $prepare, $invoke),
            ));
        };
    }
    record!(
        "warm_run_core_identity",
        &identity,
        "identity",
        "invoke",
        json!([42]),
        || (),
        |()| core.invoke("identity", &core_args)
    );
    record!(
        "warm_run_component_model_point_sum",
        &point,
        fixtures::POINT_SUM_FN,
        "invoke",
        json!([3, 4]),
        || (),
        |()| cm.invoke(fixtures::POINT_SUM_FN, &point_args)
    );
    record!(
        "cache_hit_run_core_identity",
        &identity,
        "identity",
        "Resolver::load + invoke",
        json!([42]),
        || (),
        |()| resolver
            .load(&identity)
            .and_then(|component| component.invoke("identity", &core_args))
    );
    record!(
        "cache_hit_run_component_model_point_sum",
        &point,
        fixtures::POINT_SUM_FN,
        "Resolver::load + invoke",
        json!([3, 4]),
        || (),
        |()| resolver
            .load(&point)
            .and_then(|component| component.invoke(fixtures::POINT_SUM_FN, &point_args))
    );
    record!(
        "warm_run_component_model_wasi_echo",
        &wasi,
        "echo-and-log",
        "invoke_with_wasi",
        json!({"args": [64, 0], "stdin": "hello agent-native"}),
        || WasiIoConfig::capturing().with_stdin(b"hello agent-native".to_vec()),
        |io| echo.invoke_with_wasi(&policy, &unmetered, "echo-and-log", &echo_args, io)
    );
    record!(
        "warm_run_component_model_v06_greet",
        &v06,
        "greet",
        "invoke",
        json!(["world"]),
        || (),
        |()| advanced.invoke("greet", &greet_args)
    );
    record!(
        "warm_run_component_model_v06_safe_div",
        &v06,
        "safe-div",
        "invoke",
        json!([42, 6]),
        || (),
        |()| advanced.invoke("safe-div", &div_args)
    );
    record!(
        "warm_run_core_identity_limits_unmetered",
        &identity,
        "identity",
        "invoke_with_limits(unmetered)",
        json!([42]),
        || (),
        |()| core.invoke_with_limits("identity", &core_args, &unmetered)
    );
    record!(
        "warm_run_core_identity_fuel",
        &identity,
        "identity",
        "invoke_with_limits(fuel=1000000)",
        json!([42]),
        || (),
        |()| core.invoke_with_limits("identity", &core_args, &metered)
    );
    let after = calibration(sampling.calibration);
    json!({
        "schema_version": 1, "method": "percentil_por_invocacion", "unit": "ns",
        "percentile_method": "nearest-rank; ceil(k/100*N)-1; no interpolation",
        "overhead_subtracted": false,
        "calibration": {"before": before, "after": after}, "rows": rows,
    })
}

fn arguments() -> Result<(Sampling, PathBuf), String> {
    let mut sampling = Sampling {
        n: 10_000,
        warmup: 1_000,
        calibration: 10_000,
    };
    let mut environment = None;
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        match key.as_str() {
            "--samples" => sampling.n = value.parse().map_err(|_| "invalid samples")?,
            "--warmup" => sampling.warmup = value.parse().map_err(|_| "invalid warmup")?,
            "--calibration-samples" => {
                sampling.calibration = value.parse().map_err(|_| "invalid calibration samples")?;
            }
            "--environment" => environment = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown option {key}")),
        }
    }
    if sampling.n == 0
        || sampling.n > 1_000_000
        || sampling.warmup > 1_000_000
        || sampling.calibration < 1_000
        || sampling.calibration > 1_000_000
    {
        return Err(
            "samples 1..1000000; warmup 0..1000000; calibration samples 1000..1000000".to_string(),
        );
    }
    Ok((
        sampling,
        environment.ok_or("--environment JSON path is required")?,
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sampling, path) = arguments().map_err(std::io::Error::other)?;
    let environment: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut report = collect(sampling);
    report["environment"] = environment;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_known_sequence_and_small_tail() {
        let values: Vec<_> = (1..=100).collect();
        assert_eq!(nearest_rank(&values, 50), 50);
        assert_eq!(nearest_rank(&values, 95), 95);
        assert_eq!(nearest_rank(&values, 99), 99);
        assert_eq!(nearest_rank(&(1..=10).collect::<Vec<_>>(), 99), 10);
        assert_eq!(nearest_rank(&[7], 50), 7);
    }

    #[test]
    fn summaries_sort_a_copy_and_omit_small_sample_p99() {
        let raw = vec![8, 1, 5, 3];
        let stats = statistics(&raw);
        assert_eq!((stats.min, stats.p50, stats.p95, stats.max), (1, 3, 8, 8));
        assert!(stats.p99.is_none());
        assert_eq!(raw, [8, 1, 5, 3]);
        assert_eq!(statistics(&(1..=1_000).collect::<Vec<_>>()).p99, Some(990));
    }

    #[test]
    fn measurement_does_not_subtract_empty_envelope() {
        let mut count = 0;
        let raw = measure(
            Sampling {
                n: 5,
                warmup: 3,
                calibration: 1_000,
            },
            || (),
            |()| {
                count += 1;
                Ok::<_, std::convert::Infallible>(())
            },
        );
        assert_eq!(count, 8);
        assert_eq!(raw.len(), 5);
    }
}
