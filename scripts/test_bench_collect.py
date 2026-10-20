"""Test collection failure paths and isolation without running benchmarks."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from bench_aggregate import RUNS, read_json
from bench_collect import collect
from test_bench_aggregate import environment_fixture, write_run


class CollectionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        # macOS /var aliases /private/var; Windows TEMP may use an 8.3 alias.
        self.output = Path(self.temp.name).resolve() / "bench-results"

    def test_runs_have_unique_fresh_criterion_directories(self):
        seen = []

        def command(args, **kwargs):
            if "--no-run" not in args:
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


if __name__ == "__main__":
    unittest.main()
