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
    outcomes = {}
    latencies_by_outcome = {}
    for sample in samples:
        outcome = sample["outcome"]
        outcomes[outcome] = outcomes.get(outcome, 0) + 1
        latencies_by_outcome.setdefault(outcome, []).append(sample["latency_ns"])
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
        "latency_ns_by_outcome": {
            outcome: {"p50": percentile(values, 50), "p95": percentile(values, 95),
                      "p99": percentile(values, 99), "max": max(values)}
            for outcome, values in sorted(latencies_by_outcome.items())
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


def process_tree_rss(pid):
    """Return RSS samples for a POSIX process and its descendants, in KiB."""
    if os.name != "posix":
        return {"available": False, "reason": "RSS collector currently implemented for POSIX only"}
    processes = {}
    for status in Path("/proc").glob("[0-9]*/status"):
        fields = {}
        try:
            content = status.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in content.splitlines():
            if ":" in line:
                key, value = line.split(":", 1)
                fields[key] = value.strip()
        try:
            processes[int(status.parent.name)] = (int(fields["PPid"].split()[0]),
                                                 int(fields.get("VmRSS", "0 kB").split()[0]))
        except (KeyError, ValueError):
            continue
    descendants = {pid}
    changed = True
    while changed:
        changed = False
        for candidate, (parent, _rss) in processes.items():
            if parent in descendants and candidate not in descendants:
                descendants.add(candidate)
                changed = True
    return {"available": True, "unit": "KiB", "source": "/proc/<pid>/status VmRSS",
            "value": sum(processes[item][1] for item in descendants if item in processes),
            "pids": sorted(descendants)}


def run_with_rss_samples(command, timeout_seconds=600):
    """Run one session without pipe backpressure and collect temporal RSS."""
    import tempfile

    started = time.monotonic_ns()
    with tempfile.TemporaryDirectory(prefix="worker-bench-") as temporary:
        stdout_path = Path(temporary) / "stdout.json"
        stderr_path = Path(temporary) / "stderr.log"
        with stdout_path.open("w", encoding="utf-8") as stdout, stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr:
            process = subprocess.Popen(
                command,
                cwd=REPO,
                text=True,
                encoding="utf-8",
                stdout=stdout,
                stderr=stderr,
                start_new_session=(os.name == "posix"),
            )
            samples = []
            deadline = time.monotonic() + timeout_seconds
            while process.poll() is None:
                if time.monotonic() >= deadline:
                    if os.name == "posix":
                        os.killpg(process.pid, 9)
                    else:
                        process.kill()
                    process.wait()
                    raise subprocess.TimeoutExpired(
                        command,
                        timeout_seconds,
                        output=stdout_path.read_text(encoding="utf-8"),
                        stderr=stderr_path.read_text(encoding="utf-8"),
                    )
                samples.append({"offset_ns": time.monotonic_ns() - started,
                                "rss": process_tree_rss(process.pid)})
                time.sleep(0.05)
        samples.append({"offset_ns": time.monotonic_ns() - started,
                        "rss": process_tree_rss(process.pid)})
        stdout_text = stdout_path.read_text(encoding="utf-8")
        stderr_text = stderr_path.read_text(encoding="utf-8")
    if process.returncode:
        raise subprocess.CalledProcessError(process.returncode, command, stdout_text, stderr_text)
    return stdout_text, samples
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
    command = ([str(args.benchmark_bin)] if args.benchmark_bin else [
        "cargo", "run", "--locked", "--release", "-q", "-p", "wasma-runtime",
        "--features", "experimental-workers", "--example", "worker_bench", "--"])
    command += ["--mode", mode, "--workload", workload, "--workers", str(workers),
        "--queue-capacity", str(queue_capacity), "--concurrency", str(concurrency),
        "--warmup", str(args.warmup), "--requests", str(args.requests),
    ]
    output, samples = run_with_rss_samples(command, args.session_timeout_seconds)
    return json.loads(output), {"rss_samples": samples}


def session_name(repetition, workload, workers, queue_capacity, concurrency, mode, round_number):
    return (f"r{repetition:02d}-round{round_number:04d}-{workload}-{mode}-"
            f"w{workers}-q{queue_capacity}-c{concurrency}")


def finalize_campaign(output, campaign_start, summaries):
    write_json(output / "summary.json", {
        "schema_version": 1,
        "manifest": "manifest.json",
        "campaign_elapsed_ns": time.monotonic_ns() - campaign_start,
        "sessions": summaries,
    })
    hashes = []
    for path in sorted(path for path in output.rglob("*") if path.is_file()):
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        hashes.append(f"{digest}  {path.relative_to(output).as_posix()}")
    (output / "SHA256SUMS").write_text(
        "\n".join(hashes) + "\n", encoding="utf-8", newline="\n")


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
                            for mode in (("sequential", "workers") if (round_number + repetition) % 2 else ("workers", "sequential")):
                                name = session_name(repetition, workload, workers, queue_capacity,
                                                    concurrency, mode, round_number)
                                print(f"starting {mode} {workload} w{workers} q{queue_capacity} c{concurrency}", flush=True)
                                try:
                                    document, resources = invoke(
                                        mode, workload, workers, queue_capacity, concurrency, args)
                                except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
                                    failure = Path("sessions") / name / "failure.json"
                                    write_json(output / failure, {
                                        "session": name,
                                        "type": type(error).__name__,
                                        "command": error.cmd,
                                        "timeout_seconds": getattr(error, "timeout", None),
                                        "stdout": getattr(error, "output", None),
                                        "stderr": getattr(error, "stderr", None),
                                    })
                                    finalize_campaign(output, campaign_start, summaries)
                                    raise
                                validate_document(
                                    document, mode, workload, workers, queue_capacity,
                                    concurrency, args)
                                retain_raw = (
                                    args.soak_seconds is None
                                    or len(summaries) % args.soak_raw_every == 0
                                )
                                relative = Path("sessions") / name / "requests.json"
                                if retain_raw:
                                    write_json(output / relative, document)
                                resources_relative = Path("sessions") / name / "resources.ndjson"
                                (output / resources_relative).parent.mkdir(parents=True, exist_ok=True)
                                (output / resources_relative).write_bytes(canonical({
                                    "captured_at_utc": utc_now(), "resources": resources,
                                }))
                                summary = summarize(document)
                                summary.update({"session": name,
                                                "raw": relative.as_posix() if retain_raw else None,
                                                "resources": resources_relative.as_posix()})
                                summaries.append(summary)
                                print(f"finished {name}", flush=True)
            if deadline is not None and time.monotonic() >= deadline:
                break
        if deadline is None or time.monotonic() >= deadline:
            break
    finalize_campaign(output, campaign_start, summaries)


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
    parser.add_argument("--benchmark-bin", type=Path)
    parser.add_argument("--session-timeout-seconds", type=int, default=600)
    parser.add_argument("--soak-raw-every", type=int, default=100)
    args = parser.parse_args()
    if (args.repetitions <= 0 or args.warmup < 0 or args.requests <= 0
            or args.session_timeout_seconds <= 0 or args.soak_raw_every <= 0):
        parser.error("repetitions and requests must be positive; warmup cannot be negative")
    if args.soak_seconds is not None and args.soak_seconds <= 0:
        parser.error("soak-seconds must be positive")
    if args.soak_seconds is not None and (len(args.workers) != 1 or len(args.queue_capacities) != 1
                                         or len(args.concurrency) != 1 or args.repetitions != 1):
        parser.error("soak requires one workers, queue capacity, concurrency and repetition")
    return args


if __name__ == "__main__":
    collect(arguments())
