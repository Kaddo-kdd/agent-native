"""Check archival integrity, provenance and human-certification gates offline."""

import copy
import gzip
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench_aggregate import BENCHMARKS, aggregate, write_json
from bench_evidence import load, pack, recalculate
from bench_publish import DEFAULT_INPUTS, build, check_cases, main, render_readme
from test_bench_aggregate import RUNS, environment_fixture, write_run


class EvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.scratch = tempfile.TemporaryDirectory(prefix="bench-publication-tests-")
        cls.base = Path(cls.scratch.name).resolve()
        cls.source = cls.base / "source"
        cls.source.mkdir()
        write_json(cls.source / "environment.json", environment_fixture())
        for run in RUNS:
            write_run(cls.source, run)
        write_json(cls.source / "summary.json", aggregate(cls.source))
        cls.archive = cls.base / "evidence.json.gz"
        details = pack(cls.source, cls.archive, "criterion")
        cls.entry = {"id": "123-linux", "archive": "evidence.json.gz", "kind": "criterion",
                     "run_id": "123", "attempt": "1", "commit": "a" * 40, "os": "linux",
                     "workflow_conclusion": "success", **details}
        cls.raw = cls.archive.read_bytes()
        cls.files = json.loads(gzip.decompress(cls.raw))["files"]

    @classmethod
    def tearDownClass(cls):
        cls.scratch.cleanup()

    def test_archive_is_deterministic_and_restores_raw_summary(self):
        second = self.base / "second.json.gz"
        details = pack(self.source, second, "criterion")
        self.assertEqual(second.read_bytes(), self.raw)
        self.assertEqual(details["sha256"], self.entry["sha256"])
        data = load(self.entry, self.base)
        self.assertTrue(data["criterion"]["complete"])
        self.assertEqual(len(data["criterion"]["rows"]), 14)

    def test_wasi_confirmation_archive_preserves_only_its_selected_benchmark(self):
        source = self.base / "wasi-source"
        source.mkdir()
        write_json(source / "environment.json", environment_fixture())
        for run in RUNS:
            write_run(source, run)
        selected = {"cold_run_component_model_wasi_echo": BENCHMARKS["cold_run_component_model_wasi_echo"]}
        write_json(source / "summary.json", aggregate(source, benchmarks=selected))
        archive = self.base / "wasi.json.gz"
        details = pack(source, archive, "criterion-wasi-confirmation")
        entry = dict(self.entry, id="124-linux", archive="wasi.json.gz", kind="criterion-wasi-confirmation", **details)
        data = load(entry, self.base)
        self.assertEqual([row["benchmark"] for row in data["criterion"]["rows"]], ["cold_run_component_model_wasi_echo"])

    def test_corrupt_archive_and_false_provenance_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            load(dict(self.entry, sha256="0" * 64), self.base)
        with self.assertRaisesRegex(ValueError, "provenance mismatch"):
            load(dict(self.entry, os="macos"), self.base)
        with self.assertRaisesRegex(ValueError, "payload"):
            load(dict(self.entry, payload_sha256="0" * 64), self.base)

    def test_archive_cannot_escape_its_evidence_root(self):
        for path in ("../outside.json.gz", str(self.archive), "C:/outside.json.gz"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                load(dict(self.entry, archive=path), self.base)

    def test_partial_or_quick_evidence_cannot_be_republished(self):
        files = copy.deepcopy(self.files)
        del files["run-03/collection.json"]
        with self.assertRaisesRegex(ValueError, "missing or unexpected"):
            recalculate(files, "criterion")
        files = copy.deepcopy(self.files)
        files["run-02/criterion/engine_construct/new/sample.json"] = {"iters": [1, 2], "times": [100, 200]}
        with self.assertRaisesRegex(ValueError, "Criterion evidence"):
            recalculate(files, "criterion")

    def test_changed_raw_estimate_does_not_silently_reuse_old_summary(self):
        files = copy.deepcopy(self.files)
        files["run-01/criterion/engine_construct/new/estimates.json"]["median"]["point_estimate"] = 101
        with self.assertRaisesRegex(ValueError, "stored summary differs"):
            recalculate(files, "criterion")

    def test_source_change_invalidates_curated_scenario_contract(self):
        data = load(self.entry, self.base)
        with self.assertRaisesRegex(ValueError, "scenario source changed"):
            check_cases(data, {"pinned_sources_by_os": {"linux": {"fixture": "changed"}}})

    def test_self_consistent_summary_cannot_hide_wrong_criterion_point_estimate(self):
        files = copy.deepcopy(self.files)
        files["run-01/criterion/engine_construct/new/estimates.json"]["median"]["point_estimate"] = 101
        row = files["summary.json"]["rows"][0]
        row["medians"]["run-01"] = 101
        row["range_ns"] = {"min": 100, "max": 101, "span": 1}
        row["relative_variation"] = 0.01
        with self.assertRaisesRegex(ValueError, "point estimate differs"):
            recalculate(files, "criterion")


class PublicationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.publication, cls.registry = build(DEFAULT_INPUTS)

    def test_only_human_approved_rows_receive_partial_certification(self):
        p = self.publication
        self.assertEqual(len(p["datasets"]), 12)
        self.assertEqual(len(p["rows"]), 222)
        primary = [row for row in p["rows"] if row["role"] == "primary"]
        self.assertEqual(len(primary), 69)
        self.assertFalse(p["certified"])
        certified = [row for row in p["rows"] if row["certified"]]
        self.assertEqual(len(certified), 7)
        self.assertTrue(all(row["method"] == "estimacion_agregada" for row in certified))
        self.assertTrue(all(row["provenance"]["dataset_id"] == "39449438474-windows" for row in certified))
        self.assertTrue(all(row["publication_class"] == "referencia_parcial_certificada" for row in certified))
        self.assertTrue(all(row["certification"]["supporting_dataset_id"] == "39534970668-windows" for row in certified))
        self.assertFalse(any(row["certified"] for row in p["rows"] if row["method"] == "percentil_por_invocacion"))
        unstable = [row for row in primary if row["status"] == "inestable"]
        self.assertTrue(unstable)
        self.assertTrue(all(row["publication_class"] == "excluida_inestable" for row in unstable))
        instrument = [row for row in primary if row["status"] == "aceptable" and row["instrument_flags"]]
        self.assertTrue(instrument)
        self.assertTrue(all(row["publication_class"] == "requiere_revision_instrumento" for row in instrument))

    def test_every_row_has_provenance_and_unambiguous_output_semantics(self):
        for row in self.publication["rows"]:
            p = row["provenance"]
            for key in ("commit", "os", "arch", "cpu_model", "rustc", "cargo", "wasmtime",
                        "dates_utc", "artifact_url", "archive_sha256", "payload_sha256"):
                self.assertTrue(p[key], (row["id"], key))
            self.assertEqual(len(p["commit"]), 40)
            self.assertFalse(row["case"]["outputs_observed"])
            if row["method"] == "estimacion_agregada":
                self.assertIsNone(row["wasm_sha256"])
            else:
                self.assertEqual(len(row["wasm_sha256"]), 64)
                self.assertEqual(row["descriptor"]["n"], 10_000)

    def test_publication_rejects_automatic_certification(self):
        with tempfile.TemporaryDirectory(prefix="bench-cert-gate-") as scratch:
            root = Path(scratch).resolve()
            manifest = json.loads(DEFAULT_INPUTS.read_text(encoding="utf-8"))
            manifest["certification"] = "certified"
            write_json(root / "inputs.json", manifest)
            write_json(root / "cases.json", self.registry)
            with self.assertRaisesRegex(ValueError, "automatic certification"):
                build(root / "inputs.json")

    def test_check_detects_staleness_without_overwriting(self):
        with tempfile.TemporaryDirectory(prefix="bench-stale-check-") as scratch:
            root = Path(scratch).resolve()
            path = root / "README.md"
            path.write_bytes(b"manual edit\n")
            with patch("bench_publish.outputs", return_value={"README.md": render_readme(self.publication)}), \
                    patch("sys.argv", ["bench_publish.py", "--output", str(root), "--check"]):
                with self.assertRaisesRegex(SystemExit, "Outdated"):
                    main()
            self.assertEqual(path.read_bytes(), b"manual edit\n")


if __name__ == "__main__":
    unittest.main()
