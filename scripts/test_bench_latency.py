"""Integrity checks for individual samples, instrumentation and separate p50 gating."""

import copy
import tempfile
import unittest
from pathlib import Path

from bench_aggregate import read_json, write_json
from bench_latency_aggregate import LATENCY_BENCHMARKS, aggregate_latency, nearest_rank
from test_bench_aggregate import RUNS, environment_fixture, write_run


def latency_document(run, value=1_000, n=1_000):
    def distribution(sample):
        stats = {"min": sample, "p50": sample, "p95": sample, "max": sample}
        if n >= 1_000:
            stats["p99"] = sample
        return {"n": n, "warmup": 1_000, "statistics": stats, "samples_ns": [sample] * n}

    rows = []
    for name in LATENCY_BENCHMARKS:
        rows.append(dict(distribution(value), benchmark=name, method="percentil_por_invocacion",
                         unit="ns", fixture_sha256="b" * 64, operation="identity", api="invoke",
                         arguments=[42], timed_scope="invoke only", fresh_store_per_invocation=True))
    calibration = {"n": 1_000, "warmup": 1_000, "resolution_ns": 1,
                   "samples_ns": [1] * 1_000,
                   "statistics": {"min": 1, "p50": 1, "p95": 1, "p99": 1, "max": 1}}
    return {"environment": environment_fixture(run), "method": "percentil_por_invocacion",
            "unit": "ns", "overhead_subtracted": False, "rows": rows,
            "calibration": {"before": copy.deepcopy(calibration), "after": copy.deepcopy(calibration)}}


def write_latency(root, run, document=None):
    directory = root / run
    (directory / "latency").mkdir(exist_ok=True)
    status = read_json(directory / "collection.json")
    status["commands"]["latency"] = 0
    status["latency_complete"] = True
    write_json(directory / "collection.json", status)
    write_json(directory / "latency/measurements.json", document or latency_document(run))


class LatencyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        write_json(self.root / "environment.json", environment_fixture())
        for run in RUNS:
            write_run(self.root, run)
            write_latency(self.root, run)

    def test_known_nearest_rank(self):
        self.assertEqual([nearest_rank(list(range(1, 101)), k) for k in (50, 95, 99)], [50, 95, 99])
        self.assertEqual(nearest_rank(list(range(1, 11)), 99), 10)

    def test_valid_separate_p50_and_descriptive_fuel_interval(self):
        result = aggregate_latency(self.root)
        self.assertTrue(result["complete"])
        self.assertFalse(result["certified"])
        self.assertEqual(len(result["rows"]), 9)
        self.assertTrue(all(row["status"] == "aceptable" for row in result["rows"]))
        comparison = result["fuel_comparisons"][0]
        self.assertEqual(comparison["range_ns"], {"min": 0, "max": 0})
        self.assertFalse(comparison["causal_claim"])

    def test_p50_gate_is_independent_of_criterion(self):
        for run, value in zip(RUNS, (950, 1000, 1050)):
            write_latency(self.root, run, latency_document(run, value))
        self.assertTrue(all(row["status"] == "inestable" for row in aggregate_latency(self.root)["rows"]))
        from bench_aggregate import aggregate
        self.assertTrue(all(row["status"] == "aceptable" for row in aggregate(self.root)["rows"]))

    def test_tail_variation_does_not_fail_stable_p50(self):
        doc = latency_document("run-03")
        row = doc["rows"][0]
        row["samples_ns"][-100:] = [10_000] * 100
        row["statistics"].update({"p95": 10_000, "p99": 10_000, "max": 10_000})
        write_latency(self.root, "run-03", doc)
        result = aggregate_latency(self.root)["rows"][0]
        self.assertEqual(result["status"], "aceptable")
        self.assertEqual(result["tail_ranges_ns"]["p99"], {"min": 1000, "max": 10000})
        self.assertIn("no initial 10%", result["tail_warning"])

    def test_instrument_resolution_and_overhead_warnings_are_preserved(self):
        doc = latency_document("run-01")
        doc["calibration"]["before"]["resolution_ns"] = 200
        for phase in ("before", "after"):
            cal = doc["calibration"][phase]
            cal["samples_ns"] = [150] * 1000
            cal["statistics"] = dict.fromkeys(("min", "p50", "p95", "p99", "max"), 150)
        write_latency(self.root, "run-01", doc)
        row = aggregate_latency(self.root)["rows"][0]
        self.assertIn("cerca-del-limite-de-reloj", row["instrument_warnings"]["run-01"])
        self.assertIn("overhead-al-menos-10%-del-p50", row["instrument_warnings"]["run-01"])
        self.assertEqual(row["statistics_by_run"]["run-01"]["p50"], 1000)

    def test_automatic_subtraction_and_wrong_statistics_rejected(self):
        doc = latency_document("run-01")
        doc["overhead_subtracted"] = True
        write_latency(self.root, "run-01", doc)
        self.assertFalse(aggregate_latency(self.root)["complete"])
        doc = latency_document("run-01")
        doc["rows"][0]["statistics"]["p50"] = 999
        write_latency(self.root, "run-01", doc)
        self.assertFalse(aggregate_latency(self.root)["complete"])

    def test_zero_overhead_median_is_flagged_as_unresolved(self):
        doc = latency_document("run-01")
        for phase in ("before", "after"):
            cal = doc["calibration"][phase]
            cal["samples_ns"] = [0] * 1000
            cal["statistics"] = dict.fromkeys(("min", "p50", "p95", "p99", "max"), 0)
        write_latency(self.root, "run-01", doc)
        row = aggregate_latency(self.root)["rows"][0]
        self.assertIn("mediana-de-overhead-no-resuelta-por-reloj", row["instrument_warnings"]["run-01"])

    def test_small_N_omits_p99(self):
        for run in RUNS:
            write_latency(self.root, run, latency_document(run, n=10))
        result = aggregate_latency(self.root)
        self.assertTrue(result["complete"])
        self.assertNotIn("p99", result["rows"][0]["tail_ranges_ns"])
        doc = latency_document("run-01", n=10)
        doc["rows"][0]["statistics"]["p99"] = 1000
        write_latency(self.root, "run-01", doc)
        self.assertFalse(aggregate_latency(self.root)["complete"])

    def test_missing_duplicate_and_foreign_scenarios_rejected(self):
        for mutate in (lambda doc: doc["rows"].pop(),
                       lambda doc: doc["rows"].append(copy.deepcopy(doc["rows"][0])),
                       lambda doc: doc["environment"].update(workflow_run_id="foreign")):
            doc = latency_document("run-01")
            mutate(doc)
            write_latency(self.root, "run-01", doc)
            self.assertFalse(aggregate_latency(self.root)["complete"])

    def test_changed_fixture_between_runs_is_rejected(self):
        doc = latency_document("run-03")
        doc["rows"][0]["fixture_sha256"] = "c" * 64
        write_latency(self.root, "run-03", doc)
        self.assertFalse(aggregate_latency(self.root)["complete"])

    def test_latency_failure_does_not_discard_valid_criterion_evidence(self):
        status = read_json(self.root / "run-01/collection.json")
        status.update(complete=False, criterion_complete=True, latency_complete=False)
        status["commands"]["latency"] = 1
        write_json(self.root / "run-01/collection.json", status)
        from bench_aggregate import aggregate
        self.assertTrue(aggregate(self.root)["complete"])
        self.assertFalse(aggregate_latency(self.root)["complete"])

    def test_fuel_differences_require_identical_operation_and_fixture(self):
        for run in RUNS:
            doc = latency_document(run)
            next(row for row in doc["rows"] if row["benchmark"] == "warm_run_core_identity")["fixture_sha256"] = "c" * 64
            write_latency(self.root, run, doc)
        result = aggregate_latency(self.root)
        self.assertTrue(result["complete"])
        self.assertNotIn("range_ns", result["fuel_comparisons"][0])


if __name__ == "__main__":
    unittest.main()
