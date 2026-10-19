#!/usr/bin/env python3
"""
AWS Lambda Benchmark: Rust Native vs Agent-Native WASM.

Quantifies the overhead introduced by Agent-Native (Wasmtime 27 + Component Model
+ fresh Store per invocation) against a pure native Rust baseline on AWS Lambda
(provided.al2023, x86_64, 256 MB RAM, us-west-2).

Zero third-party Python dependencies required (pure standard library).
"""

import argparse
import base64
import csv
import json
import math
import os
import re
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

REGION = "us-west-2"
ROLE_NAME = "agent-native-benchmark-role"
NATIVE_FUNCTION = "agent-native-benchmark-native"
WASM_FUNCTION = "agent-native-benchmark-wasm"

SCRIPT_DIR = Path(__file__).resolve().parent
BENCHMARK_DIR = SCRIPT_DIR.parent
ROOT_DIR = BENCHMARK_DIR.parent.parent

NATIVE_ZIP = BENCHMARK_DIR / "native" / "target" / "lambda" / "agent-native-aws-lambda-native" / "bootstrap.zip"
WASM_ZIP = ROOT_DIR / "examples" / "aws-lambda" / "target" / "lambda" / "agent-native-aws-lambda" / "bootstrap.zip"
RESULTS_DIR = BENCHMARK_DIR / "results"

EVENTS = {
    "double-value": {"operation": "double-value", "args": {"x": 21}},
    "make-point-sum": {"operation": "make-point-sum", "args": {"x": 3, "y": 4}},
}

EXPECTED_OUTPUTS = {
    "double-value": {"result": 42},
    "make-point-sum": {"result": {"sum": 7, "doubled": 14}},
}


def run_cmd(cmd: List[str], check: bool = True, capture: bool = True) -> subprocess.CompletedProcess:
    """Executes a shell command via subprocess."""
    print(f"  [EXEC] {' '.join(cmd)}")
    res = subprocess.run(cmd, capture_output=capture, text=True, check=False)
    if check and res.returncode != 0:
        err_msg = res.stderr.strip() if res.stderr else res.stdout.strip()
        raise RuntimeError(f"Command failed (exit {res.returncode}): {' '.join(cmd)}\n{err_msg}")
    return res


def get_account_id() -> str:
    res = run_cmd(["aws", "sts", "get-caller-identity", "--query", "Account", "--output", "text"])
    return res.stdout.strip()


def ensure_iam_role() -> str:
    print("\n--- Ensuring IAM Execution Role ---")
    trust_policy = {
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Principal": {"Service": "lambda.amazonaws.com"},
                "Action": "sts:AssumeRole",
            }
        ],
    }

    # Check if role exists
    res = run_cmd(["aws", "iam", "get-role", "--role-name", ROLE_NAME, "--query", "Role.Arn", "--output", "text"], check=False)
    if res.returncode == 0 and res.stdout.strip():
        role_arn = res.stdout.strip()
        print(f"Role exists: {role_arn}")
        return role_arn

    with tempfile.NamedTemporaryFile("w", delete=False, suffix=".json") as f:
        json.dump(trust_policy, f)
        trust_path = f.name

    try:
        res = run_cmd([
            "aws", "iam", "create-role",
            "--role-name", ROLE_NAME,
            "--assume-role-policy-document", f"file://{trust_path}",
            "--query", "Role.Arn", "--output", "text",
        ])
        role_arn = res.stdout.strip()
        print(f"Created role: {role_arn}")

        run_cmd([
            "aws", "iam", "attach-role-policy",
            "--role-name", ROLE_NAME,
            "--policy-arn", "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
        ])
        print("Attached AWSLambdaBasicExecutionRole policy. Waiting 10s for IAM propagation...")
        time.sleep(10)
        return role_arn
    finally:
        if os.path.exists(trust_path):
            os.remove(trust_path)


def deploy_function(function_name: str, zip_path: Path, role_arn: str) -> None:
    print(f"\n--- Deploying Function: {function_name} ---")
    if not zip_path.exists():
        raise FileNotFoundError(f"Package zip not found: {zip_path}")

    # Check if function exists
    res = run_cmd(["aws", "lambda", "get-function", "--function-name", function_name, "--region", REGION], check=False)
    if res.returncode == 0:
        print(f"Function {function_name} exists, updating code...")
        run_cmd([
            "aws", "lambda", "update-function-code",
            "--function-name", function_name,
            "--zip-file", f"fileb://{zip_path}",
            "--region", REGION,
        ])
    else:
        print(f"Creating function {function_name}...")
        run_cmd([
            "aws", "lambda", "create-function",
            "--function-name", function_name,
            "--runtime", "provided.al2023",
            "--architectures", "x86_64",
            "--role", role_arn,
            "--handler", "bootstrap",
            "--memory-size", "256",
            "--timeout", "10",
            "--zip-file", f"fileb://{zip_path}",
            "--region", REGION,
        ])

    wait_for_function_active(function_name)


def wait_for_function_active(function_name: str, max_wait_sec: int = 60) -> None:
    start = time.time()
    while time.time() - start < max_wait_sec:
        res = run_cmd([
            "aws", "lambda", "get-function-configuration",
            "--function-name", function_name,
            "--region", REGION,
            "--query", "LastUpdateStatus",
            "--output", "text",
        ], check=False)
        status = res.stdout.strip()
        if status == "Successful":
            print(f"Function {function_name} is active and ready.")
            return
        elif status == "Failed":
            raise RuntimeError(f"Function {function_name} update failed.")
        time.sleep(2)
    raise TimeoutError(f"Timed out waiting for {function_name} to become active.")


def invoke_lambda(function_name: str, payload: Dict[str, Any]) -> Tuple[Dict[str, Any], Dict[str, Any], float]:
    """Invokes Lambda with tail logs, parses report metrics, and measures client latency."""
    with tempfile.NamedTemporaryFile("w", delete=False, suffix=".json") as f:
        out_path = f.name

    payload_json = json.dumps(payload)
    client_start = time.perf_counter()

    try:
        res = run_cmd([
            "aws", "lambda", "invoke",
            "--function-name", function_name,
            "--region", REGION,
            "--payload", payload_json,
            "--cli-binary-format", "raw-in-base64-out",
            "--log-type", "Tail",
            "--query", "LogResult",
            "--output", "text",
            out_path,
        ])
        client_duration_ms = (time.perf_counter() - client_start) * 1000.0

        with open(out_path, "r", encoding="utf-8") as f:
            response_json = json.load(f)

        log_b64 = res.stdout.strip()
        logs = base64.b64decode(log_b64).decode("utf-8", errors="replace")

        metrics = parse_report_log(logs)
        metrics["client_latency_ms"] = round(client_duration_ms, 2)
        return response_json, metrics, client_duration_ms
    finally:
        if os.path.exists(out_path):
            os.remove(out_path)


def parse_report_log(logs: str) -> Dict[str, Any]:
    metrics: Dict[str, Any] = {}
    for line in logs.splitlines():
        if "REPORT RequestId:" in line:
            m_dur = re.search(r"Duration:\s*([\d\.]+)\s*ms", line)
            m_billed = re.search(r"Billed Duration:\s*([\d\.]+)\s*ms", line)
            m_mem = re.search(r"Max Memory Used:\s*(\d+)\s*MB", line)
            m_init = re.search(r"Init Duration:\s*([\d\.]+)\s*ms", line)

            if m_dur:
                metrics["duration_ms"] = float(m_dur.group(1))
            if m_billed:
                metrics["billed_duration_ms"] = float(m_billed.group(1))
            if m_mem:
                metrics["max_memory_mb"] = int(m_mem.group(1))
            if m_init:
                metrics["init_duration_ms"] = float(m_init.group(1))
    return metrics


def force_cold_start(function_name: str) -> None:
    """Updates an env variable to guarantee container recycle before next invocation."""
    salt = str(uuid.uuid4())
    run_cmd([
        "aws", "lambda", "update-function-configuration",
        "--function-name", function_name,
        "--region", REGION,
        "--environment", f"Variables={{BENCHMARK_SALT={salt}}}",
    ], capture=True)
    wait_for_function_active(function_name)


def compute_statistics(values: List[float]) -> Dict[str, float]:
    if not values:
        return {}
    sorted_vals = sorted(values)
    n = len(sorted_vals)

    def percentile(p: float) -> float:
        k = (n - 1) * p
        f = math.floor(k)
        c = math.ceil(k)
        if f == c:
            return sorted_vals[int(k)]
        return sorted_vals[f] * (c - k) + sorted_vals[c] * (k - f)

    mean = sum(sorted_vals) / n
    variance = sum((x - mean) ** 2 for x in sorted_vals) / (n - 1) if n > 1 else 0.0
    stddev = math.sqrt(variance)

    return {
        "count": n,
        "min": round(sorted_vals[0], 2),
        "max": round(sorted_vals[-1], 2),
        "mean": round(mean, 2),
        "stddev": round(stddev, 2),
        "p50": round(percentile(0.50), 2),
        "p95": round(percentile(0.95), 2),
        "p99": round(percentile(0.99), 2),
    }


def teardown_resources() -> None:
    print("\n================ TEARDOWN RESOURCES ================")
    for fn in [NATIVE_FUNCTION, WASM_FUNCTION]:
        print(f"Deleting function: {fn}...")
        run_cmd(["aws", "lambda", "delete-function", "--function-name", fn, "--region", REGION], check=False)

        log_group = f"/aws/lambda/{fn}"
        print(f"Deleting CloudWatch log group: {log_group}...")
        run_cmd(["aws", "logs", "delete-log-group", "--log-group-name", log_group, "--region", REGION], check=False)

    print(f"Detaching policy and deleting IAM role: {ROLE_NAME}...")
    run_cmd([
        "aws", "iam", "detach-role-policy",
        "--role-name", ROLE_NAME,
        "--policy-arn", "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
    ], check=False)
    run_cmd(["aws", "iam", "delete-role", "--role-name", ROLE_NAME], check=False)
    print("Teardown complete. All AWS resources deleted.")


def run_benchmark(cold_runs: int = 10, warm_runs: int = 50) -> Dict[str, Any]:
    print("\n================ STARTING BENCHMARK BATTERY ================")
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    cold_records = []
    warm_records = []

    # 1. Cold starts
    print(f"\n--- Phase 2.1: Cold Start Battery ({cold_runs} runs per function) ---")
    for i in range(1, cold_runs + 1):
        for fn_name, fn_type in [(NATIVE_FUNCTION, "native"), (WASM_FUNCTION, "wasm")]:
            print(f"Cold run #{i}/{cold_runs} for {fn_type} ({fn_name})...")
            force_cold_start(fn_name)
            resp, metrics, client_lat = invoke_lambda(fn_name, EVENTS["double-value"])
            assert resp == EXPECTED_OUTPUTS["double-value"], f"Parity mismatch in {fn_name}"

            init_dur = metrics.get("init_duration_ms")
            dur = metrics.get("duration_ms")
            billed = metrics.get("billed_duration_ms")
            mem = metrics.get("max_memory_mb")

            print(f"  -> Init: {init_dur} ms | Dur: {dur} ms | Memory: {mem} MB")
            cold_records.append({
                "run": i,
                "implementation": fn_type,
                "operation": "double-value",
                "init_duration_ms": init_dur,
                "duration_ms": dur,
                "billed_duration_ms": billed,
                "max_memory_mb": mem,
                "client_latency_ms": metrics.get("client_latency_ms"),
            })

    # 2. Warm starts (interleaved execution)
    print(f"\n--- Phase 2.2: Warm Start Battery ({warm_runs} runs per op, interleaved) ---")
    for op_name, event_payload in EVENTS.items():
        print(f"\nOperation: {op_name}")
        for i in range(1, warm_runs + 1):
            for fn_name, fn_type in [(NATIVE_FUNCTION, "native"), (WASM_FUNCTION, "wasm")]:
                resp, metrics, client_lat = invoke_lambda(fn_name, event_payload)
                assert resp == EXPECTED_OUTPUTS[op_name], f"Parity failure in {fn_type} for {op_name}: {resp}"

                warm_records.append({
                    "run": i,
                    "implementation": fn_type,
                    "operation": op_name,
                    "duration_ms": metrics.get("duration_ms"),
                    "billed_duration_ms": metrics.get("billed_duration_ms"),
                    "max_memory_mb": metrics.get("max_memory_mb"),
                    "client_latency_ms": metrics.get("client_latency_ms"),
                })
            if i % 10 == 0:
                print(f"  Progress: {i}/{warm_runs} iterations completed...")

    # Write CSVs
    cold_csv_path = RESULTS_DIR / "cold_starts.csv"
    with open(cold_csv_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=cold_records[0].keys())
        writer.writeheader()
        writer.writerows(cold_records)
    print(f"Saved cold starts to: {cold_csv_path}")

    warm_csv_path = RESULTS_DIR / "warm_invocations.csv"
    with open(warm_csv_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=warm_records[0].keys())
        writer.writeheader()
        writer.writerows(warm_records)
    print(f"Saved warm invocations to: {warm_csv_path}")

    # Compute Statistics
    summary: Dict[str, Any] = {
        "metadata": {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime()),
            "region": REGION,
            "architecture": "x86_64",
            "runtime": "provided.al2023",
            "memory_mb": 256,
            "cold_runs": cold_runs,
            "warm_runs_per_op": warm_runs,
        },
        "artifact_sizes": {
            "native": {
                "bootstrap_bytes": (BENCHMARK_DIR / "native" / "target" / "lambda" / "agent-native-aws-lambda-native" / "bootstrap").stat().st_size,
                "zip_bytes": NATIVE_ZIP.stat().st_size,
            },
            "wasm": {
                "bootstrap_bytes": (ROOT_DIR / "examples" / "aws-lambda" / "target" / "lambda" / "agent-native-aws-lambda" / "bootstrap").stat().st_size,
                "zip_bytes": WASM_ZIP.stat().st_size,
            },
        },
        "cold_starts": {},
        "warm_starts": {},
        "cost_model": {},
    }

    # Cold start stats
    for impl in ["native", "wasm"]:
        inits = [r["init_duration_ms"] for r in cold_records if r["implementation"] == impl and r["init_duration_ms"] is not None]
        mems = [r["max_memory_mb"] for r in cold_records if r["implementation"] == impl and r["max_memory_mb"] is not None]
        summary["cold_starts"][impl] = {
            "init_duration_ms": compute_statistics(inits),
            "max_memory_mb": compute_statistics(mems),
        }

    # Warm start stats
    for op in EVENTS.keys():
        summary["warm_starts"][op] = {}
        for impl in ["native", "wasm"]:
            durs = [r["duration_ms"] for r in warm_records if r["implementation"] == impl and r["operation"] == op and r["duration_ms"] is not None]
            billeds = [r["billed_duration_ms"] for r in warm_records if r["implementation"] == impl and r["operation"] == op and r["billed_duration_ms"] is not None]
            mems = [r["max_memory_mb"] for r in warm_records if r["implementation"] == impl and r["operation"] == op and r["max_memory_mb"] is not None]
            client_lats = [r["client_latency_ms"] for r in warm_records if r["implementation"] == impl and r["operation"] == op and r["client_latency_ms"] is not None]

            summary["warm_starts"][op][impl] = {
                "duration_ms": compute_statistics(durs),
                "billed_duration_ms": compute_statistics(billeds),
                "max_memory_mb": compute_statistics(mems),
                "client_latency_ms": compute_statistics(client_lats),
            }

    # Cost calculation: 256MB on us-west-2:
    # Request price: $0.20 per 1M requests ($0.0000002 / req)
    # Compute price: $0.0000166667 per GB-second => 256MB (0.25GB) = $0.000004166675 / sec = $0.000000004166675 / ms billed
    def calc_cost(invocations: int, billed_ms: float) -> float:
        req_cost = (invocations / 1_000_000.0) * 0.20
        compute_cost = invocations * billed_ms * 0.000000004166675
        return req_cost + compute_cost

    summary["cost_model"] = {
        "pricing_basis": {
            "request_per_million_usd": 0.20,
            "gb_second_usd": 0.0000166667,
            "memory_mb": 256,
        },
        "scenarios": {},
    }

    for op in EVENTS.keys():
        native_billed = summary["warm_starts"][op]["native"]["billed_duration_ms"]["mean"]
        wasm_billed = summary["warm_starts"][op]["wasm"]["billed_duration_ms"]["mean"]
        summary["cost_model"]["scenarios"][op] = {
            "per_100k": {
                "native_usd": round(calc_cost(100_000, native_billed), 4),
                "wasm_usd": round(calc_cost(100_000, wasm_billed), 4),
            },
            "per_1M": {
                "native_usd": round(calc_cost(1_000_000, native_billed), 4),
                "wasm_usd": round(calc_cost(1_000_000, wasm_billed), 4),
            },
        }

    summary_json_path = RESULTS_DIR / "benchmark_summary.json"
    with open(summary_json_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)
    print(f"\nSaved structured summary to: {summary_json_path}")

    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="AWS Lambda Benchmark Runner: Native vs Agent-Native WASM")
    parser.add_argument("--dry-run", action="store_true", help="Validate local artifacts and config without contacting AWS")
    parser.add_argument("--deploy", action="store_true", help="Deploy IAM role and both Lambda functions")
    parser.add_argument("--run", action="store_true", help="Run benchmark battery and record results")
    parser.add_argument("--teardown", action="store_true", help="Delete Lambda functions, IAM role, and CloudWatch logs")
    parser.add_argument("--cold-runs", type=int, default=10, help="Number of cold starts to measure per function (default: 10)")
    parser.add_argument("--warm-runs", type=int, default=50, help="Number of warm starts per op/function (default: 50)")
    args = parser.parse_args()

    if args.dry_run:
        print("=== DRY RUN VALIDATION ===")
        print(f"Native Zip: {NATIVE_ZIP} (exists: {NATIVE_ZIP.exists()})")
        print(f"WASM Zip:   {WASM_ZIP} (exists: {WASM_ZIP.exists()})")
        assert NATIVE_ZIP.exists(), f"Missing native zip: {NATIVE_ZIP}"
        assert WASM_ZIP.exists(), f"Missing wasm zip: {WASM_ZIP}"
        print("Local packages verified successfully.")
        return

    if args.deploy:
        role_arn = ensure_iam_role()
        deploy_function(NATIVE_FUNCTION, NATIVE_ZIP, role_arn)
        deploy_function(WASM_FUNCTION, WASM_ZIP, role_arn)
        print("\nDeployment complete.")

    if args.run:
        run_benchmark(cold_runs=args.cold_runs, warm_runs=args.warm_runs)

    if args.teardown:
        teardown_resources()

    if not (args.dry_run or args.deploy or args.run or args.teardown):
        parser.print_help()


if __name__ == "__main__":
    main()
