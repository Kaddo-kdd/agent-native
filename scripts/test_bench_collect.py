"""Test collection failure paths and isolation without running benchmarks."""

import os
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench_aggregate import RUNS, read_json
from bench_aggregate import BENCHMARKS, WASI_DIAGNOSTIC_BENCHMARKS
from bench_collect import collect, fingerprint
from test_bench_aggregate import environment_fixture, write_run
from test_bench_latency import latency_document


class CollectionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        # macOS /var aliases /private/var; Windows TEMP may use an 8.3 alias.
        self.output = Path(self.temp.name).resolve() / "bench-results"

    def test_runs_have_unique_fresh_criterion_directories(self):
        seen = []

        def command(args, **kwargs):
            if args[1] == "build":
                return subprocess.CompletedProcess(args, 0)
            if "--example" in args:
                run = Path(kwargs["env"]["CRITERION_HOME"]).parent.name
                json.dump(latency_document(run), kwargs["stdout"])
            elif "--no-run" not in args:
                home = Path(kwargs["env"]["CRITERION_HOME"])
                self.assertNotIn("CARGO_CRITERION_PORT", kwargs["env"])
                self.assertNotIn("--quick", args)
                self.assertNotIn("--test", args)
                seen.append(home)
                write_run(self.output, home.parent.name)
            return subprocess.CompletedProcess(args, 0)

        with patch("bench_collect.environment", side_effect=environment_fixture), \
                patch("bench_collect.subprocess.run", side_effect=command), \
                patch.dict(os.environ, {"CARGO_CRITERION_PORT": "1234"}):
            self.assertEqual(collect(self.output), 0)
        self.assertEqual(len(seen), 6)
        self.assertEqual(set(seen), {self.output / run / "criterion" for run in RUNS})
        for run in RUNS:
            self.assertTrue((self.output / run / "latency").is_dir())
            self.assertTrue(read_json(self.output / run / "collection.json")["complete"])
        with self.assertRaises(FileExistsError):
            collect(self.output)

    def test_failed_commands_preserve_logs_and_summary(self):
        with patch("bench_collect.environment", side_effect=environment_fixture), \
                patch("bench_collect.subprocess.run", return_value=subprocess.CompletedProcess([], 1)):
            self.assertEqual(collect(self.output), 1)
        self.assertFalse(read_json(self.output / "summary.json")["complete"])
        for run in RUNS:
            self.assertTrue((self.output / run / "baseline.log").is_file())
            self.assertFalse(read_json(self.output / run / "collection.json")["complete"])

    def test_environment_failure_still_writes_diagnostic_summary(self):
        with patch("bench_collect.environment", side_effect=ValueError("bad environment")):
            self.assertEqual(collect(self.output), 1)
        self.assertTrue((self.output / "collection-error.json").is_file())
        self.assertFalse(read_json(self.output / "summary.json")["complete"])

    def test_wasi_confirmation_collects_only_criterion_evidence(self):
        calls = []

        def command(args, **kwargs):
            calls.append(args)
            if "--no-run" not in args:
                self.assertEqual(args[-2:], ["--", "cold_run_component_model_wasi_echo"])
                write_run(self.output, Path(kwargs["env"]["CRITERION_HOME"]).parent.name)
            return subprocess.CompletedProcess(args, 0)

        selected = {"cold_run_component_model_wasi_echo": BENCHMARKS["cold_run_component_model_wasi_echo"]}
        with patch("bench_collect.environment", side_effect=environment_fixture), \
                patch("bench_collect.subprocess.run", side_effect=command):
            self.assertEqual(collect(self.output, benchmarks=selected, collect_latency=False), 0)
        summary = read_json(self.output / "summary.json")
        self.assertEqual(summary["selected_benchmarks"], ["cold_run_component_model_wasi_echo"])
        self.assertEqual([row["benchmark"] for row in summary["rows"]], ["cold_run_component_model_wasi_echo"])
        self.assertFalse((self.output / "latency-summary.json").exists())
        self.assertEqual(len(calls), 4)

    def test_wasi_diagnostic_collects_all_three_stages_without_latency(self):
        calls = []

        def command(args, **kwargs):
            calls.append(args)
            if "--no-run" not in args:
                self.assertEqual(args[-2:], ["--", "diagnostic_wasi_"])
                write_run(
                    self.output, Path(kwargs["env"]["CRITERION_HOME"]).parent.name,
                    benchmarks=WASI_DIAGNOSTIC_BENCHMARKS,
                )
            return subprocess.CompletedProcess(args, 0)

        with patch("bench_collect.environment", side_effect=environment_fixture), \
                patch("bench_collect.subprocess.run", side_effect=command):
            self.assertEqual(
                collect(
                    self.output, benchmarks=WASI_DIAGNOSTIC_BENCHMARKS, collect_latency=False,
                    filters={"wasi_diagnostic": "diagnostic_wasi_"},
                ),
                0,
            )
        summary = read_json(self.output / "summary.json")
        self.assertEqual(summary["selected_benchmarks"], list(WASI_DIAGNOSTIC_BENCHMARKS))
        self.assertEqual(len(calls), 4)
        for run in RUNS:
            status = read_json(self.output / run / "collection.json")
            self.assertIn("host_telemetry_before", status)
            self.assertIn("host_telemetry_after", status)

    def test_runtime_fingerprint_is_order_independent(self):
        hashes = {"b.rs": "b" * 64, "a.rs": "a" * 64}
        self.assertEqual(fingerprint(hashes), fingerprint(dict(reversed(list(hashes.items())))))


if __name__ == "__main__":
    unittest.main()
