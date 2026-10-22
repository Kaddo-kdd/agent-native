"""Collect isolated A/A sessions for AN-067 worker-pool evaluation.

The script never combines sessions. Each JSON result retains every request
sample, while summary.json is a deterministic view that can be recomputed from
those raw documents. It is intentionally manual: hosted CI is useful for
validation but not a performance certification environment.
"""

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


REPO = Path(__file__).resolve().parents[1]


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def canonical(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical(value))


def capture(command):
    return subprocess.check_output(command, cwd=REPO, text=True, encoding="utf-8").strip()


def locked_package_version(name):
    """Return a package version from the workspace lockfile without invoking Cargo."""
    import tomllib

    lockfile = tomllib.loads((REPO / "Cargo.lock").read_text(encoding="utf-8"))
    for package in lockfile["package"]:
        if package["name"] == name:
            return package["version"]
    raise ValueError(f"{name} was not found in Cargo.lock")


def positive_csv(value, name):
    try:
        values = sorted({int(part) for part in value.split(",")})
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"{name} must be comma-separated positive integers") from error
    if not values or any(item <= 0 for item in values):
        raise argparse.ArgumentTypeError(f"{name} must be comma-separated positive integers")
    return values


def percentile(samples, percentile):
    ordered = sorted(samples)
    return ordered[(len(ordered) * percentile + 99) // 100 - 1]


def summarize(document):
    samples = document["samples"]
    latencies = [sample["latency_ns"] for sample in samples]
    outcomes = {}
    for sample in samples:
        outcomes[sample["outcome"]] = outcomes.get(sample["outcome"], 0) + 1
    completed = outcomes.get("completed", 0)
    elapsed = document["wall_elapsed_ns"]
    return {
        "mode": document["mode"],
        "workload": document["workload"],
        "workers": document["workers"],
        "queue_capacity": document["queue_capacity"],
        "concurrency": document["concurrency"],
        "requested": document["requested"],
        "wall_elapsed_ns": elapsed,
        "throughput_per_second": completed * 1_000_000_000 / elapsed if elapsed else None,
        "outcomes": outcomes,
        "latency_ns": {
            "p50": percentile(latencies, 50),
            "p95": percentile(latencies, 95),
            "p99": percentile(latencies, 99),
            "max": max(latencies),
        },
    }


def validate_document(document, mode, workload, workers, queue_capacity, concurrency, args):
    expected = {
        "schema_version": 1,
        "mode": mode,
        "workload": workload,
        "workers": workers,
        "queue_capacity": queue_capacity,
        "concurrency": concurrency,
        "warmup": args.warmup,
        "requested": args.requests,
    }
    for field, value in expected.items():
        if document.get(field) != value:
            raise ValueError(f"benchmark document has unexpected {field!r}")
    if not isinstance(document.get("wall_elapsed_ns"), int) or document["wall_elapsed_ns"] <= 0:
        raise ValueError("benchmark document has an invalid wall_elapsed_ns")
    samples = document.get("samples")
    if not isinstance(samples, list) or len(samples) != args.requests:
        raise ValueError("benchmark document has an unexpected number of samples")
    outcomes = {"completed", "backpressure", "runtime_error", "submit_error"}
    if any(
        not isinstance(sample.get("latency_ns"), int)
        or sample["latency_ns"] <= 0
        or not isinstance(sample.get("started_offset_ns"), int)
        or not isinstance(sample.get("finished_offset_ns"), int)
        or sample["started_offset_ns"] < 0
        or sample["finished_offset_ns"] < sample["started_offset_ns"]
        or sample.get("outcome") not in outcomes
        for sample in samples
    ):
        raise ValueError("benchmark document has an invalid sample")


def rss_snapshot():
    if os.name == "posix":
        import resource

        return {"available": True, "unit": "KiB", "source": "getrusage(RUSAGE_CHILDREN).ru_maxrss",
                "value": resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss}
    return {"available": False, "reason": "RSS collector currently implemented for POSIX only"}


def environment(args):
    return {
        "schema_version": 1,
        "commit": capture(["git", "rev-parse", "HEAD"]),
        "captured_at_utc": utc_now(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "logical_cpus": os.cpu_count(),
        "python": sys.version,
        "rustc": capture(["rustc", "--version"]),
        "cargo": capture(["cargo", "--version"]),
        "wasmtime": locked_package_version("wasmtime"),
        "profile": "release",
        "feature_flags": ["experimental-workers"],
        "parameters": {
            "workers": args.workers,
            "queue_capacities": args.queue_capacities,
            "concurrency": args.concurrency,
            "workloads": args.workloads,
            "repetitions": args.repetitions,
            "warmup": args.warmup,
            "requests": args.requests,
            "soak_seconds": args.soak_seconds,
        },
    }


def invoke(mode, workload, workers, queue_capacity, concurrency, args):
    command = [
        "cargo", "run", "--locked", "--release", "-q", "-p", "agent-native-runtime",
        "--features", "experimental-workers", "--example", "worker_bench", "--",
        "--mode", mode, "--workload", workload, "--workers", str(workers),
        "--queue-capacity", str(queue_capacity), "--concurrency", str(concurrency),
        "--warmup", str(args.warmup), "--requests", str(args.requests),
    ]
    before = rss_snapshot()
    output = subprocess.check_output(command, cwd=REPO, text=True, encoding="utf-8")
    after = rss_snapshot()
    return json.loads(output), {"before": before, "after": after}


def session_name(repetition, workload, workers, queue_capacity, concurrency, mode, round_number):
    return (f"r{repetition:02d}-round{round_number:04d}-{workload}-{mode}-"
            f"w{workers}-q{queue_capacity}-c{concurrency}")


def collect(args):
    output = args.output.resolve()
    if output.exists():
        raise ValueError("output directory must not already exist")
    output.mkdir(parents=True)
    manifest = environment(args)
    write_json(output / "manifest.json", manifest)
    summaries = []
    campaign_start = time.monotonic_ns()
    deadline = time.monotonic() + args.soak_seconds if args.soak_seconds else None
    round_number = 0
    while True:
        round_number += 1
        for repetition in range(1, args.repetitions + 1):
            for workload in args.workloads:
                for workers in args.workers:
                    for queue_capacity in args.queue_capacities:
                        for concurrency in args.concurrency:
                            for mode in ("sequential", "workers"):
                                document, resources = invoke(
                                    mode, workload, workers, queue_capacity, concurrency, args)
                                validate_document(
                                    document, mode, workload, workers, queue_capacity,
                                    concurrency, args)
                                name = session_name(repetition, workload, workers, queue_capacity,
                                                    concurrency, mode, round_number)
                                relative = Path("sessions") / name / "requests.json"
                                write_json(output / relative, document)
                                resources_relative = Path("sessions") / name / "resources.ndjson"
                                (output / resources_relative).parent.mkdir(parents=True, exist_ok=True)
                                (output / resources_relative).write_bytes(canonical({
                                    "captured_at_utc": utc_now(), "resources": resources,
                                }))
                                summary = summarize(document)
                                summary.update({"session": name, "raw": relative.as_posix(),
                                                "resources": resources_relative.as_posix()})
                                summaries.append(summary)
            if deadline is not None and time.monotonic() >= deadline:
                break
        if deadline is None or time.monotonic() >= deadline:
            break
    summary = {
        "schema_version": 1,
        "manifest": "manifest.json",
        "campaign_elapsed_ns": time.monotonic_ns() - campaign_start,
        "sessions": summaries,
    }
    write_json(output / "summary.json", summary)
    hashes = []
    for path in sorted(path for path in output.rglob("*") if path.is_file()):
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        hashes.append(f"{digest}  {path.relative_to(output).as_posix()}")
    (output / "SHA256SUMS").write_text("\n".join(hashes) + "\n", encoding="utf-8", newline="\n")


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=lambda value: positive_csv(value, "workers"), required=True)
    parser.add_argument("--queue-capacities", type=lambda value: positive_csv(value, "queue capacities"), required=True)
    parser.add_argument("--concurrency", type=lambda value: positive_csv(value, "concurrency"), required=True)
    parser.add_argument("--workloads", choices=("identity", "sum"), nargs="+", default=["identity", "sum"])
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--warmup", type=int, default=1_000)
    parser.add_argument("--requests", type=int, default=10_000)
    parser.add_argument("--soak-seconds", type=int)
    args = parser.parse_args()
    if args.repetitions <= 0 or args.warmup < 0 or args.requests <= 0:
        parser.error("repetitions and requests must be positive; warmup cannot be negative")
    if args.soak_seconds is not None and args.soak_seconds <= 0:
        parser.error("soak-seconds must be positive")
    return args


if __name__ == "__main__":
    collect(arguments())
