//! Per-request A/A measurement for the experimental host-side worker pool.
//!
//! This example emits one JSON document. It intentionally does not use
//! Criterion: concurrent worker scheduling needs per-request outcomes,
//! backpressure counts, and latency samples rather than aggregated iterations.

#[path = "latency/fixtures.rs"]
mod fixtures;

use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use agent_native_runtime::workers::{ShutdownMode, WorkerPool, WorkerPoolConfig, WorkerRequest};
use agent_native_runtime::{Component, Engine, TenantConfig, Val};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Sequential,
    Workers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Workload {
    Identity,
    Sum,
}

#[derive(Debug)]
struct Config {
    mode: Mode,
    workload: Workload,
    workers: usize,
    queue_capacity: usize,
    concurrency: usize,
    warmup: usize,
    requests: usize,
}

#[derive(Serialize)]
struct Sample {
    started_offset_ns: u64,
    finished_offset_ns: u64,
    latency_ns: u64,
    outcome: &'static str,
}

#[derive(Serialize)]
struct Report {
    schema_version: u8,
    mode: &'static str,
    workload: &'static str,
    workers: usize,
    queue_capacity: usize,
    concurrency: usize,
    warmup: usize,
    requested: usize,
    wall_elapsed_ns: u64,
    samples: Vec<Sample>,
}

fn fixture(workload: Workload) -> Vec<u8> {
    match workload {
        Workload::Identity => wat::parse_str(
            r#"(module
            (func (export "identity") (param i32) (result i32) local.get 0))"#,
        )
        .expect("worker benchmark identity fixture must be valid WAT"),
        Workload::Sum => fixtures::point_sum_wasm(),
    }
}

fn function(workload: Workload) -> (&'static str, Vec<Val>) {
    match workload {
        Workload::Identity => ("identity", vec![Val::S32(42)]),
        Workload::Sum => (fixtures::POINT_SUM_FN, vec![Val::S32(3), Val::S32(4)]),
    }
}

fn tenant(engine: &Engine, index: usize) -> agent_native_runtime::Tenant {
    engine
        .create_tenant(TenantConfig::new(format!("worker-bench-{index}")).with_fuel_budget(100_000))
}

fn elapsed_ns(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).expect("elapsed time fits u64")
}

fn sequential_samples(
    engine: &Engine,
    component: &Component,
    workload: Workload,
    warmup: usize,
    requests: usize,
) -> (Vec<Sample>, u64) {
    let (function, arguments) = function(workload);
    for index in 0..warmup {
        black_box(
            tenant(engine, index)
                .invoke(component, function, &arguments)
                .expect("sequential warmup invocation"),
        );
    }
    let start = Instant::now();
    let samples = (0..requests)
        .map(|index| {
            let request_start = Instant::now();
            let started_offset_ns = elapsed_ns(start);
            let result = tenant(engine, warmup + index).invoke(component, function, &arguments);
            black_box(&result);
            let finished_offset_ns = elapsed_ns(start);
            Sample {
                started_offset_ns,
                finished_offset_ns,
                latency_ns: elapsed_ns(request_start),
                outcome: if result.is_ok() {
                    "completed"
                } else {
                    "runtime_error"
                },
            }
        })
        .collect();
    (samples, elapsed_ns(start))
}

fn worker_request(
    component: Arc<Component>,
    engine: &Engine,
    workload: Workload,
    index: usize,
) -> WorkerRequest {
    let (function, arguments) = function(workload);
    WorkerRequest::new(component, tenant(engine, index), function, arguments)
}

fn worker_samples(
    config: &Config,
    engine: &Engine,
    component: Arc<Component>,
) -> Result<(Vec<Sample>, u64), String> {
    let pool = WorkerPool::new(WorkerPoolConfig::new(config.workers, config.queue_capacity))
        .map_err(|error| error.to_string())?;
    for index in 0..config.warmup {
        let ticket = pool
            .submit(worker_request(
                Arc::clone(&component),
                engine,
                config.workload,
                index,
            ))
            .map_err(|error| error.to_string())?;
        black_box(ticket.recv().map_err(|error| error.to_string())?);
    }

    let start = Instant::now();
    let pool_ref = &pool;
    let samples = thread::scope(|scope| {
        let mut emitters = Vec::with_capacity(config.concurrency);
        for emitter in 0..config.concurrency {
            let component = Arc::clone(&component);
            emitters.push(scope.spawn(move || {
                let mut local = Vec::new();
                for index in (emitter..config.requests).step_by(config.concurrency) {
                    let request_start = Instant::now();
                    let started_offset_ns = elapsed_ns(start);
                    let outcome = match pool_ref.submit(worker_request(
                        Arc::clone(&component),
                        engine,
                        config.workload,
                        config.warmup + index,
                    )) {
                        Ok(ticket) => match ticket.recv() {
                            Ok(report) => {
                                black_box(report);
                                "completed"
                            }
                            Err(_) => "runtime_error",
                        },
                        Err(agent_native_runtime::workers::SubmitError::QueueFull { .. }) => {
                            "backpressure"
                        }
                        Err(_) => "submit_error",
                    };
                    let finished_offset_ns = elapsed_ns(start);
                    local.push(Sample {
                        started_offset_ns,
                        finished_offset_ns,
                        latency_ns: elapsed_ns(request_start),
                        outcome,
                    });
                }
                local
            }));
        }
        let mut samples = Vec::with_capacity(config.requests);
        for emitter in emitters {
            samples.extend(emitter.join().map_err(|_| "benchmark emitter panicked")?);
        }
        Ok::<_, &'static str>(samples)
    })
    .map_err(str::to_string)?;
    let elapsed = elapsed_ns(start);
    let _ = pool.close(ShutdownMode::Drain);
    Ok((samples, elapsed))
}

fn parse_usize(value: Option<String>, name: &str) -> Result<usize, String> {
    value
        .ok_or_else(|| format!("missing {name}"))?
        .parse::<usize>()
        .map_err(|_| format!("invalid {name}"))
}

fn config() -> Result<Config, String> {
    let mut values = std::collections::BTreeMap::new();
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        if values.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate option {key}"));
        }
    }
    let mode = match values.remove("--mode").as_deref() {
        Some("sequential") => Mode::Sequential,
        Some("workers") => Mode::Workers,
        _ => return Err("--mode sequential|workers is required".to_string()),
    };
    let workload = match values.remove("--workload").as_deref() {
        Some("identity") => Workload::Identity,
        Some("sum") => Workload::Sum,
        _ => return Err("--workload identity|sum is required".to_string()),
    };
    let config = Config {
        mode,
        workload,
        workers: parse_usize(values.remove("--workers"), "--workers")?,
        queue_capacity: parse_usize(values.remove("--queue-capacity"), "--queue-capacity")?,
        concurrency: parse_usize(values.remove("--concurrency"), "--concurrency")?,
        warmup: parse_usize(values.remove("--warmup"), "--warmup")?,
        requests: parse_usize(values.remove("--requests"), "--requests")?,
    };
    if !values.is_empty()
        || config.workers == 0
        || config.queue_capacity == 0
        || config.concurrency == 0
        || config.requests == 0
    {
        return Err(
            "workers, queue capacity, concurrency and requests must be positive".to_string(),
        );
    }
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config().map_err(std::io::Error::other)?;
    let engine = Engine::new()?;
    let component = Arc::new(engine.load_component(&fixture(config.workload))?);
    let (samples, wall_elapsed_ns) = match config.mode {
        Mode::Sequential => sequential_samples(
            &engine,
            &component,
            config.workload,
            config.warmup,
            config.requests,
        ),
        Mode::Workers => {
            worker_samples(&config, &engine, component).map_err(std::io::Error::other)?
        }
    };
    let report = Report {
        schema_version: 1,
        mode: match config.mode {
            Mode::Sequential => "sequential",
            Mode::Workers => "workers",
        },
        workload: match config.workload {
            Workload::Identity => "identity",
            Workload::Sum => "sum",
        },
        workers: config.workers,
        queue_capacity: config.queue_capacity,
        concurrency: config.concurrency,
        warmup: config.warmup,
        requested: config.requests,
        wall_elapsed_ns,
        samples,
    };
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
