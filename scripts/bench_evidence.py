"""Archive the JSON inputs needed to reproduce benchmark summaries offline.

Hashes establish integrity of the recorded files, not performance certification.
Plots and logs remain in the original GitHub artifact; raw measurement inputs do not expire here.
"""

import argparse
import gzip
import hashlib
import io
import json
import math
import statistics
import tempfile
from pathlib import Path, PurePosixPath

from bench_aggregate import BENCHMARKS, RUNS, aggregate, read_json, write_json
from bench_latency_aggregate import aggregate_latency


WASI_CONFIRMATION = {"cold_run_component_model_wasi_echo": BENCHMARKS["cold_run_component_model_wasi_echo"]}


def configuration(kind):
    if kind == "criterion":
        return BENCHMARKS, False
    if kind == "criterion-and-latency":
        return BENCHMARKS, True
    if kind == "criterion-wasi-confirmation":
        return WASI_CONFIRMATION, False
    raise ValueError("unknown evidence kind: " + kind)


def canonical(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n").encode("utf-8")


def sha256(value):
    return hashlib.sha256(value).hexdigest()


def members(kind):
    benchmarks, has_latency = configuration(kind)
    names = {"environment.json", "summary.json"}
    for run in RUNS:
        names.update({run + "/environment.json", run + "/collection.json"})
        for name in benchmarks:
            for file in ("benchmark.json", "sample.json", "estimates.json"):
                names.add(run + "/criterion/" + name + "/new/" + file)
        if has_latency:
            names.add(run + "/latency/measurements.json")
    if has_latency:
        names.add("latency-summary.json")
    return names


def recalculate(files, kind):
    # Exact allowlist prevents archive entries escaping the fresh staging folder.
    if set(files) != members(kind):
        raise ValueError("evidence has missing or unexpected members")
    with tempfile.TemporaryDirectory(prefix="bench-evidence-") as scratch:
        root = Path(scratch).resolve()
        if not root.is_relative_to(Path(tempfile.gettempdir()).resolve()):
            raise ValueError("staging directory outside the intended temporary directory")
        for name, value in files.items():
            destination = root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            write_json(destination, value)
        benchmarks, has_latency = configuration(kind)
        criterion = aggregate(root, benchmarks=benchmarks if kind == "criterion-wasi-confirmation" else None)
        if criterion != files["summary.json"] or any(len(row["medians"]) != 3 for row in criterion["rows"]):
            raise ValueError("Criterion evidence incomplete or stored summary differs from raw samples")
        # Criterion 0.5 WallTime estimates the median of time/iteration samples.
        # Independently verify the point estimate; do not re-run its bootstrap CIs.
        for run in RUNS:
            for name in benchmarks:
                prefix = run + "/criterion/" + name + "/new/"
                samples = files[prefix + "sample.json"]
                median = statistics.median(time / count for time, count in zip(samples["times"], samples["iters"]))
                recorded = files[prefix + "estimates.json"]["median"]["point_estimate"]
                if not math.isclose(median, recorded, rel_tol=1e-12, abs_tol=1e-9):
                    raise ValueError("Criterion point estimate differs from raw time/iteration median")
        latency = None
        if has_latency:
            latency = aggregate_latency(root)
            if latency != files["latency-summary.json"] or any(len(row["statistics_by_run"]) != 3 for row in latency["rows"]):
                raise ValueError("individual evidence incomplete or stored summary differs from raw samples")
    return criterion, latency


def pack(root, destination, kind):
    files = {name: read_json(root / name) for name in sorted(members(kind))}
    recalculate(files, kind)
    payload = canonical({"schema_version": 1, "kind": kind, "files": files})
    buffer = io.BytesIO()
    # Fixed timestamp and empty filename avoid machine/time dependent gzip headers.
    with gzip.GzipFile(fileobj=buffer, mode="wb", filename="", mtime=0, compresslevel=9) as stream:
        stream.write(payload)
    compressed = buffer.getvalue()
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(compressed)
    return {"sha256": sha256(compressed), "payload_sha256": sha256(payload),
            "compressed_bytes": len(compressed), "members": len(files)}


def load(entry, root):
    relative = PurePosixPath(entry["archive"])
    if relative.is_absolute() or ".." in relative.parts or "\\" in entry["archive"] or ":" in entry["archive"]:
        raise ValueError("archive must use a portable relative path")
    destination = (root / entry["archive"]).resolve()
    if not destination.is_relative_to(root.resolve()):
        raise ValueError("archive path outside evidence root")
    compressed = destination.read_bytes()
    if len(compressed) != entry["compressed_bytes"] or sha256(compressed) != entry["sha256"]:
        raise ValueError(entry["id"] + ": compressed evidence hash mismatch")
    with gzip.GzipFile(fileobj=io.BytesIO(compressed), mode="rb") as stream:
        payload = stream.read(64 * 1024 * 1024 + 1)
    if len(payload) > 64 * 1024 * 1024 or sha256(payload) != entry["payload_sha256"]:
        raise ValueError(entry["id"] + ": payload size or hash mismatch")
    document = json.loads(payload)
    if document.get("schema_version") != 1 or document.get("kind") != entry["kind"]:
        raise ValueError("unsupported or mismatched evidence schema")
    if len(document.get("files", {})) != entry["members"]:
        raise ValueError("evidence member count mismatch")
    for path in ["environment.json"] + [run + "/environment.json" for run in RUNS]:
        if document["files"][path].get("measurement_mode") != "criterion-default-full":
            raise ValueError("quick or unverified measurement mode is not publishable")
    criterion, latency = recalculate(document["files"], entry["kind"])
    env = criterion["environment"]
    for expected, field in (("run_id", "workflow_run_id"), ("attempt", "workflow_run_attempt"),
                            ("commit", "commit"), ("os", "os")):
        if str(entry[expected]) != str(env[field]):
            raise ValueError(entry["id"] + ": provenance mismatch: " + field)
    if entry.get("workflow_conclusion") != "success":
        raise ValueError("workflow evidence was not recorded as complete")
    return {"entry": entry, "environment": env, "criterion": criterion, "latency": latency,
            "files": document["files"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--kind", choices=("criterion", "criterion-and-latency", "criterion-wasi-confirmation"), required=True)
    args = parser.parse_args()
    print(json.dumps(pack(args.root, args.destination, args.kind)))


if __name__ == "__main__":
    main()
