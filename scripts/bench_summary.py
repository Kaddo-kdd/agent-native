"""Expose technical evidence in the artifact and GitHub job summary, without certification."""

import argparse
import json
import os
from pathlib import Path


def load(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return {"rows": [], "errors": [str(exc)]}


def microseconds(value):
    return "{:.3f}".format(value / 1000) if value is not None else "pendiente"


def range_text(value):
    return microseconds(value["min"]) + "–" + microseconds(value["max"]) if value else "omitido"


def render(root):
    criterion = load(root / "summary.json")
    latency = load(root / "latency-summary.json")
    env = criterion.get("environment", {})
    lines = ["# Resultados técnicos de benchmarks — sin certificar", "",
             "OS: **" + env.get("os", "pendiente") + "**. CPU: " + env.get("cpu_model", "pendiente") + ".",
             "Commit: `" + env.get("commit", "pendiente") + "`.", "",
             "Cada grupo contiene tres repeticiones en el mismo runner. No se agrupan ejecuciones independientes.",
             "Un workflow verde indica recolección completa; las filas inestables se conservan y no certifican una línea base.", "",
             "## Criterion: medianas agregadas (µs)", "",
             "Estas estimaciones no son percentiles de invocaciones individuales.", "",
             "| Benchmark | R1 | R2 | R3 | Variación | Estado |", "|---|---:|---:|---:|---:|---|"]
    for row in criterion.get("rows", []):
        medians = row.get("medians", {})
        cells = [microseconds(medians.get(run)) for run in ("run-01", "run-02", "run-03")]
        variation = row.get("relative_variation")
        cells += ["{:.2f} %".format(variation * 100) if variation is not None else "no evaluada", row["status"]]
        lines.append("| `" + row["benchmark"] + "` | " + " | ".join(cells) + " |")
    lines += ["", "## Invocaciones individuales: p50 y rangos de colas (µs)", "",
              "Nearest-rank. Umbral < 10 % únicamente para p50; p95/p99 conservan rangos y advertencias de variabilidad.",
              "Preparación de entradas y destrucción de resultados fuera del intervalo. Store nuevo por invocación.", "",
              "| Benchmark | p50 R1 | p50 R2 | p50 R3 | Variación p50 | p95 min–max | p99 min–max | Estado | Advertencias |",
              "|---|---:|---:|---:|---:|---:|---:|---|---|"]
    for row in latency.get("rows", []):
        values = row.get("statistics_by_run", {})
        cells = [microseconds(values.get(run, {}).get("p50")) for run in ("run-01", "run-02", "run-03")]
        variation = row.get("relative_variation_p50")
        tails = row.get("tail_ranges_ns", {})
        flags = sorted({flag for warnings in row.get("instrument_warnings", {}).values() for flag in warnings})
        cells += ["{:.2f} %".format(variation * 100) if variation is not None else "no evaluada",
                  range_text(tails.get("p95")), range_text(tails.get("p99")), row["status"], ", ".join(flags) or "—"]
        lines.append("| `" + row["benchmark"] + "` | " + " | ".join(cells) + " |")
    lines += ["", "## Instrumento y límites", "",
              "No se resta overhead de ninguna muestra. Una mediana de overhead de 0 ns puede estar por debajo de la resolución del reloj; no demuestra coste cero.",
              "La resolución observada es el mínimo salto positivo entre lecturas, una cota empírica superior de la resolución intrínseca.",
              "Ver muestras, calibración antes/después y comparaciones de fuel en los JSON del artefacto. Los intervalos de diferencias de fuel son rangos observados, no intervalos de confianza.",
              "Cambiar de identity a WASI echo o IR v0.6 cambia la operación; no aísla por sí solo la sobrecarga de una característica."]
    errors = criterion.get("errors", []) + latency.get("errors", [])
    if errors:
        lines += ["", "## Evidencia incompleta", ""] + ["- " + error for error in errors]
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    args.root.mkdir(parents=True, exist_ok=True)
    report = render(args.root)
    (args.root / "REPORT.md").write_text(report, encoding="utf-8")
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a", encoding="utf-8") as stream:
            stream.write(report)


if __name__ == "__main__":
    main()
