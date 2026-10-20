"""Validate AN-053 evidence and summarise Criterion medians (Python stdlib only).

No individual-invocation percentiles or baseline certification are inferred here.
"""

import argparse
import json
import math
import statistics
from pathlib import Path


RUNS = ("run-01", "run-02", "run-03")
# Explicit inventory catches missing benches, even if absent in all three runs.
BENCHMARKS = {
    "engine_construct": ("baseline", "engine", "engine", "construct"),
    "load_core_identity": ("baseline", "load", "core_identity", "identity"),
    "load_component_model_point_sum": ("baseline", "load", "cm_point_sum", "make-point-sum"),
    "cold_run_core_identity": ("baseline", "cold_run", "core_identity", "identity"),
    "cold_run_component_model_point_sum": ("baseline", "cold_run", "cm_point_sum", "make-point-sum"),
    "warm_run_core_identity": ("baseline", "warm_run", "core_identity", "identity"),
    "warm_run_component_model_point_sum": ("baseline", "warm_run", "cm_point_sum", "make-point-sum"),
    "cache_hit_run_core_identity": ("baseline", "cache_hit_run", "core_identity", "identity"),
    "cache_hit_run_component_model_point_sum": ("baseline", "cache_hit_run", "cm_point_sum", "make-point-sum"),
    "warm_run_component_model_wasi_echo": ("extended", "warm_run", "cm_wasi_echo", "echo-and-log"),
    "cold_run_component_model_wasi_echo": ("extended", "cold_run", "cm_wasi_echo", "echo-and-log"),
    "warm_run_component_model_v06_greet": ("extended", "warm_run", "cm_v06", "greet"),
    "warm_run_component_model_v06_safe_div": ("extended", "warm_run", "cm_v06", "safe-div"),
    "warm_run_core_identity_fuel": ("extended", "warm_run", "core_identity", "identity"),
}
ENV_KEYS = (
    "os", "arch", "cpu_model", "logical_cpus", "rustc", "cargo", "wasmtime",
    "profile", "commit", "workflow_run_id", "workflow_run_attempt", "runner_os",
    "source_sha256", "measurement_mode",
)


def read_json(path):
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def positive(value):
    return type(value) in (int, float) and math.isfinite(value) and value > 0


def environment_errors(env, run=None):
    errors = []
    for key in ENV_KEYS:
        value = env.get(key)
        if not value or (isinstance(value, str) and value.strip().lower() in ("", "unknown")):
            errors.append("missing environment field: " + key)
    if env.get("measurement_mode") != "criterion-default-full":
        errors.append("full Criterion settings not established")
    if env.get("tracked_dirty") is not False:
        errors.append("tracked source tree is dirty or unverified")
    if not env.get("captured_at_utc"):
        errors.append("missing capture timestamp")
    if run is not None and env.get("repetition") != run:
        errors.append("repetition identity mismatch")
    return errors


def aggregate(root):
    root = Path(root)
    errors = []
    environments = {}
    try:
        reference = read_json(root / "environment.json")
        errors.extend(environment_errors(reference))
    except (OSError, ValueError, TypeError, AttributeError) as exc:
        reference = {}
        errors.append("root environment: " + str(exc))
    for run in RUNS:
        try:
            env = read_json(root / run / "environment.json")
            errors.extend(run + ": " + error for error in environment_errors(env, run))
            environments[run] = env
        except (OSError, ValueError, TypeError, AttributeError) as exc:
            errors.append(run + " environment: " + str(exc))

    mixed_hardware = any(
        env.get("cpu_model") != reference.get("cpu_model")
        for env in environments.values()
    )
    for run, env in environments.items():
        for key in ENV_KEYS:
            if env.get(key) != reference.get(key):
                errors.append(run + ": environment mismatch: " + key)

    rows = []
    for name, (suite, stage, fixture, operation) in BENCHMARKS.items():
        medians = {}
        sources = {}
        sample_counts = {}
        row_errors = list(errors)
        for run in RUNS:
            directory = root / run / "criterion" / name / "new"
            try:
                status = read_json(root / run / "collection.json")
                if status.get("commands", {}).get(suite) != 0 or status.get("criterion_complete", status.get("complete")) is not True:
                    raise ValueError("collection incomplete or benchmark command failed")
                identity = read_json(directory / "benchmark.json")
                if identity["full_id"] != name:
                    raise ValueError("benchmark identity mismatch")
                samples = read_json(directory / "sample.json")
                iters, times = samples["iters"], samples["times"]
                if len(iters) < 100 or len(iters) != len(times):
                    raise ValueError("expected >=100 Criterion samples (quick/incomplete data rejected)")
                if not all(positive(value) for value in iters + times):
                    raise ValueError("invalid Criterion samples")
                median = read_json(directory / "estimates.json")["median"]["point_estimate"]
                if not positive(median):
                    raise ValueError("median must be finite and positive")
                medians[run] = median
                sample_counts[run] = len(iters)
                sources[run] = (directory / "estimates.json").relative_to(root).as_posix()
            except (OSError, ValueError, TypeError, KeyError, AttributeError) as exc:
                row_errors.append(run + ": " + str(exc))

        row = {
            "benchmark": name, "suite": suite, "stage": stage, "fixture": fixture,
            "operation": operation, "method": "estimacion_agregada",
            "indicator": "criterion_median", "unit": "ns", "medians": medians,
            "criterion_sample_counts": sample_counts, "sources": sources,
            "status": "incompleto", "reasons": row_errors,
            "certified": False,
        }
        if mixed_hardware:
            row["status"] = "hardware-mixto"
        elif not row_errors and len(medians) == 3:
            values = list(medians.values())
            centre = statistics.median(values)
            span = max(values) - min(values)
            variation = span / centre
            row.update({
                "median_of_medians_ns": centre,
                "range_ns": {"min": min(values), "max": max(values), "span": span},
                "relative_variation": variation,
                "status": "aceptable" if variation < 0.10 else "inestable",
                "below_10_us": min(values) < 10_000,
            })
        rows.append(row)

    complete = not errors and all(row["status"] in ("aceptable", "inestable") for row in rows)
    return {
        "schema_version": 1, "scope": "same-job-repeatability",
        "environment": reference, "runs": list(RUNS), "complete": complete,
        "certified": False,
        "rule": "(max - min) / median of three Criterion medians < 0.10",
        "errors": errors, "rows": rows,
        "limitations": [
            "GitHub-hosted runner scenarios; not universal OS comparisons.",
            "Same-runner repeatability does not establish reproducibility on another machine.",
            "Criterion medians are aggregate estimates, not individual latency p50.",
            "Different operations do not isolate WASI or IR v0.6 execution overhead.",
            "Unstable rows remain technical evidence; baseline certification requires human review.",
        ],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    summary = aggregate(args.root)
    write_json(args.root / "summary.json", summary)
    return 0 if summary["complete"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
