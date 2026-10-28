"""Run three full Criterion and individual-latency repetitions with isolated evidence.

Usage: python scripts/bench_collect.py [--output bench-results]
No quick mode, pooled workflow data, or publication/certification step.
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from bench_aggregate import BENCHMARKS, RUNS, WASI_DIAGNOSTIC_BENCHMARKS, aggregate, write_json
from bench_latency_aggregate import aggregate_latency


REPO = Path(__file__).resolve().parents[1]
SOURCES = (
    "Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "crates/runtime/Cargo.toml",
    "crates/runtime/benches/baseline.rs", "crates/runtime/benches/extended.rs",
    "crates/runtime/benches/wasi_diagnostic.rs",
    "crates/runtime/benches/fixtures/wasi_console_echo.wat",
    "crates/compiler/tests/fixtures/ir/program-005.ir.json",
    "crates/runtime/examples/bench_env.rs", "scripts/bench_collect.py",
    "scripts/bench_aggregate.py", ".github/workflows/bench.yml",
    "crates/runtime/examples/bench_latency.rs",
    "crates/runtime/examples/latency/fixtures.rs", "scripts/bench_latency_aggregate.py",
    "scripts/bench_summary.py",
)


def runtime_sources():
    """Return the workspace files that can affect the runtime executable."""
    roots = ("crates/runtime/src", "crates/ir/src")
    fixed = ("Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "crates/ir/Cargo.toml", "crates/runtime/Cargo.toml")
    discovered = [
        path.relative_to(REPO).as_posix()
        for root in roots
        for path in (REPO / root).rglob("*.rs")
    ]
    return tuple(sorted((*fixed, *discovered)))


def source_hashes(names):
    return {name: hashlib.sha256((REPO / name).read_bytes()).hexdigest() for name in names}


def fingerprint(hashes):
    digest = hashlib.sha256()
    for name, value in sorted(hashes.items()):
        digest.update(name.encode("utf-8"))
        digest.update(b"\0")
        digest.update(value.encode("ascii"))
        digest.update(b"\n")
    return digest.hexdigest()


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def capture(command):
    return subprocess.check_output(command, cwd=REPO, text=True, encoding="utf-8").strip()


def environment(run=None):
    env = json.loads(capture(["cargo", "run", "--locked", "-q", "-p", "wasma-runtime", "--example", "bench_env"]))
    runtime_hashes = source_hashes(runtime_sources())
    env.update({
        "commit": capture(["git", "rev-parse", "HEAD"]),
        "tracked_dirty": bool(capture(["git", "status", "--porcelain", "--untracked-files=no"])),
        "captured_at_utc": utc_now(),
        "workflow_run_id": os.environ.get("GITHUB_RUN_ID", "local"),
        "workflow_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", "1"),
        "workflow_url": (os.environ.get("GITHUB_SERVER_URL", "https://github.com") + "/"
                         + os.environ.get("GITHUB_REPOSITORY", "local") + "/actions/runs/"
                         + os.environ.get("GITHUB_RUN_ID", "local")),
        "runner_os": os.environ.get("RUNNER_OS", sys.platform),
        "runner_image": os.environ.get("ImageOS", "local"),
        "runner_image_version": os.environ.get("ImageVersion", "local"),
        "measurement_mode": "criterion-default-full",
        "source_sha256": source_hashes(SOURCES),
        "runtime_source_sha256": runtime_hashes,
        "runtime_source_fingerprint_sha256": fingerprint(runtime_hashes),
    })
    if run:
        env["repetition"] = run
    return env


def collect(output, benchmarks=None, collect_latency=True, filters=None):
    benchmarks = BENCHMARKS if benchmarks is None else benchmarks
    # Refuse to reuse an evidence directory, even after a partial/failed run.
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    # Pre-create skeletons so failed collections still expose missing evidence.
    for run in RUNS:
        (output / run / "criterion").mkdir(parents=True)
        (output / run / "latency").mkdir()
    failed = False
    try:
        write_json(output / "environment.json", environment())
        # Compile once outside the measured repetitions; never use --quick.
        suites = tuple(sorted({value[0] for value in benchmarks.values()}))
        compile_command = ["cargo", "bench", "--locked", "-p", "wasma-runtime"]
        for suite in suites:
            compile_command.extend(("--bench", suite))
        subprocess.run([*compile_command, "--no-run"], cwd=REPO, check=True)
        if collect_latency:
            subprocess.run([
                "cargo", "build", "--release", "--locked", "-p", "wasma-runtime",
                "--example", "bench_latency",
            ], cwd=REPO, check=True)
        for run in RUNS:
            directory = output / run
            status = {"started_at_utc": utc_now(), "complete": False, "commands": {}}
            write_json(directory / "collection.json", status)
            try:
                run_environment = environment(run)
                status["host_telemetry_before"] = run_environment.get("host_telemetry")
                write_json(directory / "environment.json", run_environment)
                child_env = os.environ.copy()
                # Criterion 0.5 honours CRITERION_HOME; fresh output directories
                # prevent both overwrites and stale results from cargo caches.
                child_env["CRITERION_HOME"] = str(directory / "criterion")
                # cargo-criterion's optional socket must not redirect evidence.
                child_env.pop("CARGO_CRITERION_PORT", None)
                for suite in suites:
                    command = ["cargo", "bench", "--locked", "-p", "wasma-runtime", "--bench", suite]
                    selected = [name for name, value in benchmarks.items() if value[0] == suite]
                    if len(benchmarks) != len(BENCHMARKS):
                        command.extend(("--", (filters or {}).get(suite, selected[0])))
                    print(run + ": " + " ".join(command), flush=True)
                    with (directory / (suite + ".log")).open("w", encoding="utf-8") as log:
                        result = subprocess.run(command, cwd=REPO, env=child_env, stdout=log, stderr=subprocess.STDOUT)
                    status["commands"][suite] = result.returncode
                    write_json(directory / "collection.json", status)
                status["criterion_complete"] = all(status["commands"].get(suite) == 0 for suite in suites)
                if collect_latency:
                    command = ["cargo", "run", "--release", "--locked", "-q", "-p", "wasma-runtime",
                               "--example", "bench_latency", "--", "--environment", str(directory / "environment.json")]
                    print(run + ": individual invocation latencies", flush=True)
                    with (directory / "latency/measurements.json").open("w", encoding="utf-8") as output_stream, \
                            (directory / "latency.log").open("w", encoding="utf-8") as log:
                        result = subprocess.run(command, cwd=REPO, env=child_env, stdout=output_stream, stderr=log)
                    status["commands"]["latency"] = result.returncode
                    status["latency_complete"] = result.returncode == 0
                status["complete"] = all(code == 0 for code in status["commands"].values())
                failed = failed or not status["complete"]
            except (OSError, ValueError, subprocess.SubprocessError) as exc:
                status["error"] = str(exc)
                failed = True
            finally:
                # This snapshot is deliberately outside Criterion's timed scope.
                # A missing probe is evidence too, so it is represented as null.
                try:
                    status["host_telemetry_after"] = environment(run).get("host_telemetry")
                except (OSError, ValueError, subprocess.SubprocessError) as exc:
                    status["host_telemetry_after_error"] = str(exc)
                status["finished_at_utc"] = utc_now()
                write_json(directory / "collection.json", status)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        write_json(output / "collection-error.json", {"error": str(exc), "at_utc": utc_now()})
        print(str(exc), file=sys.stderr)
        failed = True
    finally:
        summary = aggregate(output, benchmarks=benchmarks)
        write_json(output / "summary.json", summary)
        latency_summary = aggregate_latency(output) if collect_latency else None
        if latency_summary is not None:
            write_json(output / "latency-summary.json", latency_summary)
        for row in summary["rows"]:
            print(row["benchmark"] + ": " + row["status"], flush=True)
    # Noise is data: instability does not fail collection. Missing/invalid evidence does.
    return 1 if failed or not summary["complete"] or (latency_summary is not None and not latency_summary["complete"]) else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=REPO / "bench-results")
    parser.add_argument("--scope", choices=("full", "wasi-confirmation", "wasi-diagnostic"), default="full")
    args = parser.parse_args()
    if args.scope == "wasi-confirmation":
        selected = {"cold_run_component_model_wasi_echo": BENCHMARKS["cold_run_component_model_wasi_echo"]}
        return collect(args.output, benchmarks=selected, collect_latency=False)
    if args.scope == "wasi-diagnostic":
        return collect(
            args.output, benchmarks=WASI_DIAGNOSTIC_BENCHMARKS, collect_latency=False,
            filters={"wasi_diagnostic": "diagnostic_wasi_"},
        )
    return collect(args.output)


if __name__ == "__main__":
    raise SystemExit(main())
