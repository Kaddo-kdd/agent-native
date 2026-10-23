import importlib.util
import json
import tempfile
import sys
import subprocess
import unittest
from types import SimpleNamespace
from unittest.mock import patch
from pathlib import Path


SPEC = importlib.util.spec_from_file_location(
    "worker_bench_collect", Path(__file__).with_name("worker_bench_collect.py"))
worker_bench_collect = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(worker_bench_collect)


class WorkerBenchCollectTests(unittest.TestCase):
    def test_nearest_rank_summary_preserves_outcomes(self):
        document = {
            "mode": "workers", "workload": "identity", "workers": 2,
            "queue_capacity": 2, "concurrency": 4, "requested": 4,
            "wall_elapsed_ns": 1000,
            "samples": [
                {"started_offset_ns": 0, "finished_offset_ns": 4, "latency_ns": 4, "outcome": "completed"},
                {"started_offset_ns": 4, "finished_offset_ns": 5, "latency_ns": 1, "outcome": "backpressure"},
                {"started_offset_ns": 5, "finished_offset_ns": 7, "latency_ns": 2, "outcome": "completed"},
                {"started_offset_ns": 7, "finished_offset_ns": 10, "latency_ns": 3, "outcome": "runtime_error"},
            ],
        }
        summary = worker_bench_collect.summarize(document)
        self.assertEqual(summary["outcomes"], {"completed": 2, "backpressure": 1, "runtime_error": 1})
        self.assertEqual(summary["latency_ns_by_outcome"], {
            "backpressure": {"p50": 1, "p95": 1, "p99": 1, "max": 1},
            "completed": {"p50": 2, "p95": 4, "p99": 4, "max": 4},
            "runtime_error": {"p50": 3, "p95": 3, "p99": 3, "max": 3},
        })
        self.assertEqual(summary["throughput_per_second"], 2_000_000)

    def test_rss_runner_records_relative_samples_and_exits(self):
        output, samples = worker_bench_collect.run_with_rss_samples(
            [sys.executable, "-c", "print('rss-test')"])
        self.assertEqual(output.strip(), "rss-test")
        self.assertTrue(samples)
        self.assertEqual(samples, sorted(samples, key=lambda sample: sample["offset_ns"]))
        for sample in samples:
            self.assertIn("rss", sample)
            self.assertIn("available", sample["rss"])

    def test_rss_runner_handles_large_output_without_pipe_blocking(self):
        output, _samples = worker_bench_collect.run_with_rss_samples(
            [sys.executable, "-c", "print('x' * 200000)"], timeout_seconds=5)
        self.assertEqual(len(output.strip()), 200000)

    def test_rss_runner_enforces_timeout(self):
        with self.assertRaises(subprocess.TimeoutExpired):
            worker_bench_collect.run_with_rss_samples(
                [sys.executable, "-c", "import time; time.sleep(2)"], timeout_seconds=0.05)
    def test_positive_csv_deduplicates_and_rejects_zero(self):
        self.assertEqual(worker_bench_collect.positive_csv("4,1,4", "workers"), [1, 4])
        with self.assertRaises(Exception):
            worker_bench_collect.positive_csv("0,1", "workers")

    def test_validate_document_rejects_mismatched_config(self):
        args = SimpleNamespace(warmup=1, requests=2)
        document = {
            "schema_version": 1, "mode": "workers", "workload": "identity",
            "workers": 2, "queue_capacity": 2, "concurrency": 2,
            "warmup": 1, "requested": 2, "wall_elapsed_ns": 10,
            "samples": [
                {"started_offset_ns": 0, "finished_offset_ns": 2, "latency_ns": 2, "outcome": "completed"},
                {"started_offset_ns": 3, "finished_offset_ns": 6, "latency_ns": 3, "outcome": "backpressure"},
            ],
        }
        worker_bench_collect.validate_document(document, "workers", "identity", 2, 2, 2, args)
        document["workers"] = 1
        with self.assertRaises(ValueError):
            worker_bench_collect.validate_document(document, "workers", "identity", 2, 2, 2, args)

    def test_collect_writes_self_contained_hashed_session(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "worker-results"
            args = SimpleNamespace(
                output=output, workers=[1], queue_capacities=[1], concurrency=[1],
                workloads=["identity"], repetitions=1, warmup=0, requests=1,
                soak_seconds=None, soak_raw_every=100,
            )

            def invoke(mode, workload, workers, queue_capacity, concurrency, _args):
                return ({
                    "schema_version": 1, "mode": mode, "workload": workload,
                    "workers": workers, "queue_capacity": queue_capacity,
                    "concurrency": concurrency, "warmup": 0, "requested": 1,
                    "wall_elapsed_ns": 5,
                    "samples": [{
                        "started_offset_ns": 0, "finished_offset_ns": 5,
                        "latency_ns": 5, "outcome": "completed",
                    }],
                }, {"before": {"available": False}, "after": {"available": False}})

            with patch.object(worker_bench_collect, "environment", return_value={"schema_version": 1}), \
                    patch.object(worker_bench_collect, "invoke", side_effect=invoke):
                worker_bench_collect.collect(args)

            summary = json.loads((output / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(len(summary["sessions"]), 2)
            self.assertTrue((output / "SHA256SUMS").read_text(encoding="utf-8"))
            for session in summary["sessions"]:
                self.assertTrue((output / session["raw"]).is_file())
                self.assertTrue((output / session["resources"]).is_file())

    def test_collect_preserves_failure_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "worker-results"
            args = SimpleNamespace(
                output=output, workers=[1], queue_capacities=[1], concurrency=[1],
                workloads=["identity"], repetitions=1, warmup=0, requests=1,
                soak_seconds=None, soak_raw_every=100,
            )
            failure = subprocess.TimeoutExpired(["benchmark"], 1, output="partial")
            with patch.object(worker_bench_collect, "environment", return_value={"schema_version": 1}), \
                    patch.object(worker_bench_collect, "invoke", side_effect=failure):
                with self.assertRaises(subprocess.TimeoutExpired):
                    worker_bench_collect.collect(args)
            self.assertTrue((output / "summary.json").is_file())
            self.assertTrue((output / "SHA256SUMS").is_file())
            self.assertTrue(list((output / "sessions").rglob("failure.json")))


if __name__ == "__main__":
    unittest.main()
