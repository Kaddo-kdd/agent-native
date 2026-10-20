"""Acceptance tests for evidence integrity and the approved stability rule."""

import tempfile
import unittest
from pathlib import Path

from bench_aggregate import BENCHMARKS, RUNS, aggregate, write_json


def environment_fixture(run=None):
    env = {
        "os": "linux", "arch": "x86_64", "cpu_model": "Test CPU",
        "logical_cpus": 4, "rustc": "rustc 1.85.0", "cargo": "cargo 1.85.0",
        "wasmtime": "29.0.1", "profile": "release (criterion)",
        "commit": "a" * 40, "workflow_run_id": "123", "workflow_run_attempt": "1",
        "runner_os": "Linux", "source_sha256": {"fixture": "b" * 64},
        "measurement_mode": "criterion-default-full", "tracked_dirty": False,
        "captured_at_utc": "2026-10-20T00:00:00+00:00",
    }
    if run:
        env["repetition"] = run
    return env


def write_run(root, run, value=100.0):
    directory = root / run
    directory.mkdir(parents=True, exist_ok=True)
    write_json(directory / "environment.json", environment_fixture(run))
    write_json(directory / "collection.json", {"complete": True, "commands": {"baseline": 0, "extended": 0}})
    for name in BENCHMARKS:
        raw = directory / "criterion" / name / "new"
        raw.mkdir(parents=True, exist_ok=True)
        write_json(raw / "benchmark.json", {"full_id": name})
        write_json(raw / "sample.json", {"iters": [10] * 100, "times": [value * 10] * 100})
        write_json(raw / "estimates.json", {"median": {"point_estimate": value}})


class AggregateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        write_json(self.root / "environment.json", environment_fixture())
        for run in RUNS:
            write_run(self.root, run)

    def row(self, name="engine_construct"):
        return next(row for row in aggregate(self.root)["rows"] if row["benchmark"] == name)

    def test_stable_complete_preserves_three_medians_and_raw_links(self):
        summary = aggregate(self.root)
        self.assertTrue(summary["complete"])
        self.assertEqual(len(summary["rows"]), 14)
        self.assertFalse(summary["certified"])
        row = self.row()
        self.assertEqual(row["status"], "aceptable")
        self.assertEqual(row["relative_variation"], 0)
        self.assertEqual(row["criterion_sample_counts"], dict.fromkeys(RUNS, 100))
        self.assertEqual(len(row["medians"]), 3)
        for source in row["sources"].values():
            self.assertTrue((self.root / source).is_file())
        self.assertEqual(row["method"], "estimacion_agregada")
        self.assertNotIn("p50", row)
        self.assertTrue(row["below_10_us"])

    def test_exact_ten_percent_is_unstable_but_complete(self):
        for run, value in zip(RUNS, (95, 100, 105)):
            write_run(self.root, run, value)
        row = self.row()
        self.assertEqual(row["status"], "inestable")
        self.assertEqual(row["range_ns"], {"min": 95, "max": 105, "span": 10})
        self.assertEqual(row["relative_variation"], 0.1)
        self.assertTrue(aggregate(self.root)["complete"])

    def test_variation_uses_median_of_medians(self):
        for run, value in zip(RUNS, (100, 101, 109)):
            write_run(self.root, run, value)
        self.assertAlmostEqual(self.row()["relative_variation"], 9 / 101)
        self.assertEqual(self.row()["status"], "aceptable")

    def test_missing_benchmark_in_all_runs_is_not_silently_omitted(self):
        for run in RUNS:
            (self.root / run / "criterion/engine_construct/new/estimates.json").unlink()
        self.assertEqual(self.row()["status"], "incompleto")
        self.assertFalse(aggregate(self.root)["complete"])

    def test_mixed_cpu_never_evaluates_variation(self):
        env = environment_fixture("run-02")
        env["cpu_model"] = "Other CPU"
        write_json(self.root / "run-02/environment.json", env)
        self.assertEqual(self.row()["status"], "hardware-mixto")
        self.assertNotIn("relative_variation", self.row())

    def test_unknown_cpu_and_foreign_workflow_are_rejected(self):
        for key, value in (("cpu_model", "unknown"), ("workflow_run_id", "456"),
                           ("workflow_run_attempt", "2"), ("commit", "c" * 40),
                           ("tracked_dirty", True), ("source_sha256", {"fixture": "changed"})):
            with self.subTest(key=key):
                env = environment_fixture("run-02")
                env[key] = value
                write_json(self.root / "run-02/environment.json", env)
                self.assertFalse(aggregate(self.root)["complete"])
                self.assertNotIn("relative_variation", self.row())
        write_json(self.root / "run-02/environment.json", environment_fixture("run-02"))

    def test_quick_and_malformed_evidence_rejected(self):
        raw = self.root / "run-01/criterion/engine_construct/new"
        write_json(raw / "sample.json", {"iters": [1, 2], "times": [100, 200]})
        self.assertEqual(self.row()["status"], "incompleto")
        write_run(self.root, "run-01")
        write_json(raw / "benchmark.json", {"full_id": "different_benchmark"})
        self.assertEqual(self.row()["status"], "incompleto")

    def test_nonfinite_or_nonpositive_medians_rejected(self):
        target = self.root / "run-01/criterion/engine_construct/new/estimates.json"
        for value in ("NaN", "Infinity", "0", "-1", "true", '"100"'):
            with self.subTest(value=value):
                target.write_text('{"median":{"point_estimate":' + value + '}}', encoding="utf-8")
                self.assertEqual(self.row()["status"], "incompleto")

    def test_command_failure_invalidates_evidence_even_if_files_exist(self):
        write_json(self.root / "run-01/collection.json", {"complete": False, "commands": {"baseline": 1, "extended": 0}})
        self.assertEqual(self.row()["status"], "incompleto")

    def test_missing_metadata_never_certifies(self):
        (self.root / "run-03/environment.json").unlink()
        self.assertFalse(aggregate(self.root)["complete"])
        self.assertEqual(self.row()["status"], "incompleto")


if __name__ == "__main__":
    unittest.main()
