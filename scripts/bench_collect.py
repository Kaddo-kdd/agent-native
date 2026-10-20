"""Run three full Criterion repetitions in one runner with isolated evidence.

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

from bench_aggregate import RUNS, aggregate, write_json


REPO = Path(__file__).resolve().parents[1]
SOURCES = (
    "Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "crates/runtime/Cargo.toml",
    "crates/runtime/benches/baseline.rs", "crates/runtime/benches/extended.rs",
    "crates/runtime/benches/fixtures/wasi_console_echo.wat",
    "crates/compiler/tests/fixtures/ir/program-005.ir.json",
    "crates/runtime/examples/bench_env.rs", "scripts/bench_collect.py",
    "scripts/bench_aggregate.py", ".github/workflows/bench.yml",
)


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def capture(command):
    return subprocess.check_output(command, cwd=REPO, text=True, encoding="utf-8").strip()


def environment(run=None):
    env = json.loads(capture(["cargo", "run", "--locked", "-q", "-p", "agent-native-runtime", "--example", "bench_env"]))
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
        "source_sha256": {
            name: hashlib.sha256((REPO / name).read_bytes()).hexdigest() for name in SOURCES
        },
    })
    if run:
        env["repetition"] = run
    return env


def collect(output):
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
        subprocess.run([
            "cargo", "bench", "--locked", "-p", "agent-native-runtime",
            "--bench", "baseline", "--bench", "extended", "--no-run",
        ], cwd=REPO, check=True)
        for run in RUNS:
            directory = output / run
            status = {"started_at_utc": utc_now(), "complete": False, "commands": {}}
            write_json(directory / "collection.json", status)
            try:
                write_json(directory / "environment.json", environment(run))
                child_env = os.environ.copy()
                # Criterion 0.5 honours CRITERION_HOME; fresh output directories
                # prevent both overwrites and stale results from cargo caches.
                child_env["CRITERION_HOME"] = str(directory / "criterion")
                # cargo-criterion's optional socket must not redirect evidence.
                child_env.pop("CARGO_CRITERION_PORT", None)
                for suite in ("baseline", "extended"):
                    command = ["cargo", "bench", "--locked", "-p", "agent-native-runtime", "--bench", suite]
                    print(run + ": " + " ".join(command), flush=True)
                    with (directory / (suite + ".log")).open("w", encoding="utf-8") as log:
                        result = subprocess.run(command, cwd=REPO, env=child_env, stdout=log, stderr=subprocess.STDOUT)
                    status["commands"][suite] = result.returncode
                    write_json(directory / "collection.json", status)
                status["complete"] = all(code == 0 for code in status["commands"].values())
                failed = failed or not status["complete"]
            except (OSError, ValueError, subprocess.SubprocessError) as exc:
                status["error"] = str(exc)
                failed = True
            finally:
                status["finished_at_utc"] = utc_now()
                write_json(directory / "collection.json", status)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        write_json(output / "collection-error.json", {"error": str(exc), "at_utc": utc_now()})
        print(str(exc), file=sys.stderr)
        failed = True
    finally:
        summary = aggregate(output)
        write_json(output / "summary.json", summary)
        for row in summary["rows"]:
            print(row["benchmark"] + ": " + row["status"], flush=True)
    # Noise is data: instability does not fail collection. Missing/invalid evidence does.
    return 1 if failed or not summary["complete"] else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=REPO / "bench-results")
    args = parser.parse_args()
    return collect(args.output)


if __name__ == "__main__":
    raise SystemExit(main())
