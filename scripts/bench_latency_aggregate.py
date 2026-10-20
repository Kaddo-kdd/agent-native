"""Validate raw individual latencies; assess p50 separately from Criterion."""

import argparse
import statistics
from pathlib import Path

from bench_aggregate import BENCHMARKS, RUNS, aggregate, read_json, write_json


LATENCY_BENCHMARKS = tuple(
    name for name in BENCHMARKS if name.startswith(("warm_run_", "cache_hit_run_"))
) + ("warm_run_core_identity_limits_unmetered",)


def nearest_rank(values, percentile):
    ordered = sorted(values)
    return ordered[(len(ordered) * percentile + 99) // 100 - 1]


def validate_distribution(value):
    samples = value["samples_ns"]
    n = value["n"]
    if type(n) is not int or n < 1 or n > 1_000_000 or len(samples) != n:
        raise ValueError("invalid sample count")
    if any(type(sample) is not int or sample < 0 for sample in samples):
        raise ValueError("latencies must be nonnegative integer nanoseconds")
    expected = {"min": min(samples), "p50": nearest_rank(samples, 50),
                "p95": nearest_rank(samples, 95), "max": max(samples)}
    if n >= 1_000:
        expected["p99"] = nearest_rank(samples, 99)
    if value["statistics"] != expected:
        raise ValueError("statistics do not match raw samples/nearest-rank or p99 suppression")
    return expected


def validate_calibration(value):
    overhead = []
    resolutions = []
    for phase in ("before", "after"):
        calibration = value[phase]
        stats = validate_distribution(calibration)
        if calibration["n"] < 1_000 or calibration.get("warmup") != 1_000:
            raise ValueError("insufficient instrument calibration")
        resolution = calibration["resolution_ns"]
        if resolution is not None and (type(resolution) is not int or resolution <= 0):
            raise ValueError("invalid clock resolution")
        resolutions.append(resolution)
        overhead.append(stats["p50"])
    # A missing resolution on either side of the session remains unknown.
    resolution = max(resolutions) if None not in resolutions else None
    return resolution, max(overhead)


def quality_warnings(p50, resolution, overhead, n, warmup):
    warnings = []
    if resolution is None:
        warnings.append("resolucion-no-observada")
    elif p50 < 10 * resolution:
        warnings.append("cerca-del-limite-de-reloj")
    if p50 == 0 or overhead * 10 >= p50:
        warnings.append("overhead-al-menos-10%-del-p50")
    if overhead == 0:
        warnings.append("mediana-de-overhead-no-resuelta-por-reloj")
    if n < 1_000:
        warnings.append("p99-omitido-por-N-menor-a-1000")
    if warmup < 1_000:
        warnings.append("warmup-menor-a-1000")
    return warnings


def aggregate_latency(root):
    root = Path(root)
    criterion = aggregate(root)
    errors = list(criterion["errors"])
    reference = criterion["environment"]
    documents = {}
    calibrations = {}
    for run in RUNS:
        try:
            directory = root / run
            status = read_json(directory / "collection.json")
            if status.get("commands", {}).get("latency") != 0 or status.get("latency_complete") is not True:
                raise ValueError("latency collection incomplete or failed")
            document = read_json(directory / "latency/measurements.json")
            expected_env = read_json(directory / "environment.json")
            if document["environment"] != expected_env:
                raise ValueError("latency environment does not match repetition")
            if document.get("method") != "percentil_por_invocacion" or document.get("unit") != "ns":
                raise ValueError("not individual invocation latencies in ns")
            if document.get("overhead_subtracted") is not False:
                raise ValueError("automatic overhead subtraction is not permitted")
            names = [row["benchmark"] for row in document["rows"]]
            if len(names) != len(set(names)) or set(names) != set(LATENCY_BENCHMARKS):
                raise ValueError("missing, duplicate or unexpected scenarios")
            calibrations[run] = validate_calibration(document["calibration"])
            documents[run] = {row["benchmark"]: row for row in document["rows"]}
        except (OSError, ValueError, TypeError, KeyError, AttributeError) as exc:
            errors.append(run + ": " + str(exc))

    mixed = any("environment mismatch: cpu_model" in error for error in errors)
    rows = []
    for name in LATENCY_BENCHMARKS:
        values = {}
        counts = {}
        flags = {}
        sources = {}
        reasons = list(errors)
        identity = None
        descriptor = {}
        for run in RUNS:
            try:
                row = documents[run][name]
                if row["method"] != "percentil_por_invocacion" or row["unit"] != "ns":
                    raise ValueError("invalid row method or unit")
                if row.get("fresh_store_per_invocation") is not True:
                    raise ValueError("fresh Store contract missing")
                warmup = row["warmup"]
                if type(warmup) is not int or warmup < 0:
                    raise ValueError("invalid warmup")
                stats = validate_distribution(row)
                fingerprint = row["fixture_sha256"]
                if len(fingerprint) != 64 or any(c not in "0123456789abcdef" for c in fingerprint):
                    raise ValueError("invalid fixture fingerprint")
                current_identity = {key: row[key] for key in (
                    "fixture_sha256", "operation", "api", "arguments", "timed_scope", "n", "warmup")}
                if identity is not None and identity != current_identity:
                    raise ValueError("fixture, operation or sampling differs between repetitions")
                identity = current_identity
                descriptor = current_identity
                values[run] = stats
                counts[run] = row["n"]
                resolution, overhead = calibrations[run]
                flags[run] = quality_warnings(stats["p50"], resolution, overhead, row["n"], warmup)
                sources[run] = (Path(run) / "latency/measurements.json").as_posix()
            except (OSError, ValueError, TypeError, KeyError, AttributeError) as exc:
                reasons.append(run + ": " + str(exc))

        output = {"benchmark": name, "method": "percentil_por_invocacion", "unit": "ns",
                  "indicator": "individual_latency_p50", "statistics_by_run": values,
                  "sample_counts": counts, "instrument_warnings": flags, "sources": sources,
                  "descriptor": descriptor, "status": "incompleto", "reasons": reasons, "certified": False}
        if mixed:
            output["status"] = "hardware-mixto"
        elif not reasons and len(values) == 3:
            p50s = [stats["p50"] for stats in values.values()]
            centre = statistics.median(p50s)
            if centre == 0:
                output["status"] = "precision-insuficiente"
            else:
                variation = (max(p50s) - min(p50s)) / centre
                output.update({"status": "aceptable" if variation < 0.10 else "inestable",
                               "relative_variation_p50": variation,
                               "p50_range_ns": {"min": min(p50s), "max": max(p50s)}})
            output["tail_ranges_ns"] = {
                key: {"min": min(stats[key] for stats in values.values()),
                      "max": max(stats[key] for stats in values.values())}
                for key in ("p95", "p99") if all(key in stats for stats in values.values())
            }
            output["tail_warning"] = "ranges across repetitions; tails are variable, no initial 10% threshold"
        rows.append(output)

    comparisons = []
    by_name = {row["benchmark"]: row for row in rows}
    fuel = by_name["warm_run_core_identity_fuel"]
    for control_name in ("warm_run_core_identity", "warm_run_core_identity_limits_unmetered"):
        control = by_name[control_name]
        comparison = {
            "treated": fuel["benchmark"], "control": control_name,
            "method": "difference_of_individual_p50_per_repetition", "unit": "ns",
            "certified": False, "causal_claim": False,
            "interpretation": "same-operation API-path difference" if control_name == "warm_run_core_identity" else "same-operation and same-API fuel comparison",
            "interval_kind": "observed range of three paired p50 differences, not a confidence interval",
        }
        same_fixture = all(fuel["descriptor"].get(key) == control["descriptor"].get(key)
                           for key in ("fixture_sha256", "operation", "arguments"))
        if not fuel["reasons"] and not control["reasons"] and same_fixture and all(
                run in fuel["statistics_by_run"] and run in control["statistics_by_run"] for run in RUNS):
            differences = {run: fuel["statistics_by_run"][run]["p50"] - control["statistics_by_run"][run]["p50"] for run in RUNS}
            comparison.update({"differences_by_run": differences,
                               "median_difference_ns": statistics.median(differences.values()),
                               "range_ns": {"min": min(differences.values()), "max": max(differences.values())},
                               "both_p50_stable": fuel["status"] == control["status"] == "aceptable",
                               "instrument_warnings": {"treated": fuel["instrument_warnings"], "control": control["instrument_warnings"]}})
        comparisons.append(comparison)

    return {"schema_version": 1, "environment": reference, "method": "percentil_por_invocacion",
            "calibration_by_run": {run: {"resolution_ns": value[0], "overhead_p50_ns": value[1]}
                                   for run, value in calibrations.items()},
            "complete": not errors and all(row["status"] in ("aceptable", "inestable", "precision-insuficiente") for row in rows),
            "certified": False, "errors": errors, "rows": rows, "fuel_comparisons": comparisons,
            "limitations": ["Criterion medians and individual p50 are evaluated separately.",
                            "No automatic overhead subtraction; inspect clock/overhead warnings before interpretation.",
                            "p95/p99 have observed ranges and variability warnings, no initial 10% gate.",
                            "Scenarios run in fixed order; observed fuel differences can include temporal drift and are not causal estimates.",
                            "Different WASI/IR operations are scenarios, not isolated execution overhead."]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    result = aggregate_latency(args.root)
    write_json(args.root / "latency-summary.json", result)
    return 0 if result["complete"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
