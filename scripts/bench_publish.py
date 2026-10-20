"""Regenerate the AN-055 candidate publication from archived evidence, without network.

Run from any directory: python scripts/bench_publish.py [--check]
There is deliberately no automatic performance-certification option.
"""

import argparse
import copy
import json
from collections import Counter
from pathlib import Path

from bench_aggregate import BENCHMARKS, RUNS, read_json
from bench_evidence import load
from bench_latency_aggregate import LATENCY_BENCHMARKS


DEFAULT_INPUTS = Path(__file__).resolve().parents[1] / "docs/benchmarks/inputs.json"
OS_NAMES = {"linux": "Linux", "windows": "Windows", "macos": "macOS"}
METHODS = ("estimacion_agregada", "percentil_por_invocacion")


def compact(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def cell(value):
    return str(value).replace("|", "\\|").replace("\n", " ")


def us(value):
    return "{:.3f}".format(value / 1000) if value is not None else "no evaluada"


def percent(value):
    return "{:.2f} %".format(value * 100) if value is not None else "no evaluada"


def interval(value):
    return us(value["min"]) + "–" + us(value["max"]) if value else "no disponible"


def classification(status, flags):
    if status != "aceptable":
        return "excluida_" + status
    return "requiere_revision_instrumento" if flags else "candidata_revision_humana"


def editorial(row):
    if row["status"] != "aceptable":
        return "Excluida: " + row["status"]
    return "Revisar instrumento" if row["instrument_flags"] else "Candidata a revisión"


def status_counts(rows):
    counts = Counter(row["status"] for row in rows)
    return "; ".join(str(value) + " " + name for name, value in sorted(counts.items()))


def provenance(dataset):
    entry, env, files = dataset["entry"], dataset["environment"], dataset["files"]
    return {
        "dataset_id": entry["id"], "commit": env["commit"], "os": env["os"],
        "arch": env["arch"], "cpu_model": env["cpu_model"], "rustc": env["rustc"],
        "cargo": env["cargo"], "wasmtime": env["wasmtime"], "profile": env["profile"],
        "session_captured_at_utc": env["captured_at_utc"],
        "repetition_captured_at_utc": {run: files[run + "/environment.json"]["captured_at_utc"] for run in RUNS},
        "cpu_models_by_run": {run: files[run + "/environment.json"]["cpu_model"] for run in RUNS},
        "collection_periods_utc": {run: {key: files[run + "/collection.json"].get(key)
                                           for key in ("started_at_utc", "finished_at_utc")} for run in RUNS},
        "dates_utc": sorted({files[run + "/environment.json"]["captured_at_utc"].split("T")[0] for run in RUNS}),
        "workflow_url": entry["workflow_url"], "artifact_url": entry["artifact_url"],
        "artifact_name": entry["artifact_name"], "archive": entry["archive"],
        "archive_sha256": entry["sha256"], "payload_sha256": entry["payload_sha256"],
    }


def check_cases(dataset, registry):
    expected = registry["pinned_sources_by_os"][dataset["environment"]["os"]]
    # Historical datasets predate the individual collector. Do not invent its data.
    paths = list(expected) if dataset["latency"] else [path for path in expected if "/examples/" not in path]
    for run in RUNS:
        recorded = dataset["files"][run + "/environment.json"]["source_sha256"]
        for path in paths:
            if recorded.get(path) != expected[path]:
                raise ValueError(dataset["entry"]["id"] + ": scenario source changed: " + path)
    if dataset["latency"]:
        for row in dataset["latency"]["rows"]:
            case = registry["scenarios"][row["benchmark"]]
            descriptor = row["descriptor"]
            for field in ("operation", "arguments"):
                if case[field] != descriptor[field]:
                    raise ValueError("scenario contract and individual evidence disagree: " + field)
            if case["individual_api"] != descriptor["api"] or case["individual_timed_scope"] != descriptor["timed_scope"]:
                raise ValueError("scenario API or timing boundary differs from evidence")


def rows_for(dataset, registry):
    result = []
    common = provenance(dataset)
    for method, summary in ((METHODS[0], dataset["criterion"]), (METHODS[1], dataset["latency"])):
        if summary is None:
            continue
        for measured in summary["rows"]:
            row = copy.deepcopy(measured)
            row["id"] = dataset["entry"]["id"] + "/" + method + "/" + row["benchmark"]
            row["role"] = dataset["entry"]["role"]
            row["provenance"] = common
            row["case"] = registry["scenarios"][row["benchmark"]]
            row["certified"] = False
            flags = sorted({flag for group in row.get("instrument_warnings", {}).values() for flag in group})
            row["instrument_flags"] = flags
            row["publication_class"] = classification(row["status"], flags)
            row["exclusion_reason"] = None if row["status"] == "aceptable" else row["status"]
            if method == METHODS[0]:
                row["sample_type"] = "Criterion grouped statistical samples; not individual invocations"
                row["total_iterations_by_run"] = {
                    run: sum(dataset["files"][run + "/criterion/" + row["benchmark"] + "/new/sample.json"]["iters"])
                    for run in RUNS
                }
                # These runs did not record the compiled WASM hash for Criterion.
                row["wasm_sha256"] = None
                row["wasm_hash_note"] = "Not recorded by Criterion; trace fixture via pinned sources and measured commit."
            else:
                row["sample_type"] = "individual invocation durations"
                row["wasm_sha256"] = row["descriptor"]["fixture_sha256"]
            result.append(row)
    return result


def build(inputs_path):
    root = inputs_path.parent
    manifest = read_json(inputs_path)
    registry = read_json(root / "cases.json")
    if manifest.get("schema_version") != 1 or manifest.get("certification") != "pending-human-review":
        raise ValueError("unsupported manifest or automatic certification requested")
    if registry.get("schema_version") != 1:
        raise ValueError("unsupported scenario registry")
    if set(registry["scenarios"]) != set(BENCHMARKS) | set(LATENCY_BENCHMARKS):
        raise ValueError("scenario inventory differs from benchmark inventory")
    datasets, seen = [], set()
    for entry in manifest["datasets"]:
        if entry["id"] != str(entry["run_id"]) + "-" + entry["os"] or entry["id"] in seen:
            raise ValueError("duplicate or mismatched dataset identity")
        seen.add(entry["id"])
        if entry["role"] not in ("primary", "historical", "verification") or entry["os"] not in OS_NAMES:
            raise ValueError("unsupported dataset role or OS")
        if entry["artifact_url"] != entry["workflow_url"] + "/artifacts/" + str(entry["artifact_id"]):
            raise ValueError("artifact provenance URL inconsistent")
        data = load(entry, root)
        if entry["workflow_url"] != data["environment"]["workflow_url"]:
            raise ValueError("workflow URL differs from recorded environment")
        check_cases(data, registry)
        datasets.append(data)
    datasets.sort(key=lambda item: item["entry"]["id"])
    primary = [data for data in datasets if data["entry"]["role"] == "primary"]
    if len(primary) != 3 or {data["environment"]["os"] for data in primary} != set(OS_NAMES):
        raise ValueError("primary evidence must contain exactly one dataset per OS")
    if any(data["entry"]["run_id"] != manifest["primary_run_id"] or data["latency"] is None for data in primary):
        raise ValueError("primary workflow identity or individual evidence missing")
    rows = [row for dataset in datasets for row in rows_for(dataset, registry)]
    result = {
        "schema_version": 1, "certification": "pending-human-review", "certified": False,
        "primary_run_id": manifest["primary_run_id"],
        "selection_rule": "complete AN-054 workflow fixed in inputs.json; no selection by performance or stability",
        "units": "ns", "datasets": [{**data["entry"], "environment": data["environment"]} for data in datasets],
        "rows": rows,
        "calibration": {data["entry"]["id"]: data["latency"]["calibration_by_run"] for data in primary},
        "fuel_comparisons": {data["entry"]["id"]: data["latency"]["fuel_comparisons"] for data in primary},
        "individual_reproducibility": "not established: one independent individual-collector workflow",
        "limitations": [
            "Escenarios de runners hospedados, sin rendimiento universal por OS ni SLA.",
            "Repetibilidad en el mismo runner y reproducibilidad entre workflows son evaluaciones separadas.",
            "Las medianas Criterion no son p50 de invocaciones individuales.",
            "p95/p99 conservan rangos y advertencias de variabilidad, sin umbral inicial del 10 %.",
            "No se resta overhead; el salto de reloj observado no es su resolución intrínseca.",
            "Las salidas esperadas describen contratos; no son valores devueltos archivados.",
            "Operaciones distintas no aíslan la sobrecarga de WASI, IR v0.6 ni del runtime.",
            "Los intervalos de fuel son rangos observados, no de confianza; el orden fijo puede introducir deriva temporal.",
            "No hay comparaciones con implementaciones nativas ni con el arranque de AWS Lambda.",
        ],
    }
    if sum(row["role"] == "primary" for row in rows) != 3 * (len(BENCHMARKS) + len(LATENCY_BENCHMARKS)):
        raise ValueError("primary publication lost or duplicated rows")
    comparisons = []
    for os in OS_NAMES:
        before = [data for data in datasets if data["entry"]["os"] == os and data["entry"]["role"] == "historical"]
        after = [data for data in datasets if data["entry"]["os"] == os and data["entry"]["role"] == "verification"]
        for a in before:
            for b in after:
                comparisons.append({"os": os, "before": a["entry"]["id"], "after": b["entry"]["id"],
                                    "same_recorded_sources": a["environment"]["source_sha256"] == b["environment"]["source_sha256"],
                                    "same_cpu_model": a["environment"]["cpu_model"] == b["environment"]["cpu_model"],
                                    "same_image": all(a["environment"][key] == b["environment"][key] for key in ("runner_image", "runner_image_version")),
                                    "samples_pooled": False})
    result["historical_comparisons"] = comparisons
    return result, registry


def table(headers, values):
    return ["| " + " | ".join(headers) + " |", "|" + "---|" * len(headers)] + [
        "| " + " | ".join(cell(value) for value in row) + " |" for row in values
    ]


def context(env):
    return ["", "| Campo | Evidencia |", "|---|---|",
            "| Commit medido | `" + env["commit"] + "` |",
            "| Captura UTC | " + env["captured_at_utc"] + " (ficha de sesión, no timestamp de cada muestra) |",
            "| OS / arquitectura | " + env["os"] + " / " + env["arch"] + " |",
            "| CPU / lógicas disponibles | " + env["cpu_model"] + " / " + str(env["logical_cpus"]) + " |",
            "| Rust | " + env["rustc"] + " |", "| Cargo | " + env["cargo"] + " |",
            "| Wasmtime / perfil | " + env["wasmtime"] + " / " + env["profile"] + " |",
            "| Imagen | " + env["runner_image"] + " / " + env["runner_image_version"] + " |", ""]


def measurement_tables(rows):
    lines = ["### Criterion — estimacion_agregada", "", "Medianas en µs; N cuenta muestras agrupadas Criterion, no invocaciones individuales.", ""]
    criterion = [row for row in rows if row["method"] == METHODS[0]]
    lines += table(["Benchmark / fixture", "R1 µs", "R2 µs", "R3 µs", "Variación", "N por R", "Estado editorial", "Procedencia"], [
        ["`" + row["benchmark"] + "` / " + row["case"]["fixture"],
         *[us(row["medians"].get(run)) for run in RUNS], percent(row.get("relative_variation")),
         "/".join(str(row["criterion_sample_counts"].get(run, "?")) for run in RUNS),
         editorial(row), "[" + row["provenance"]["dataset_id"] + "](" + row["provenance"]["artifact_url"] + ")"]
        for row in criterion])
    individual = [row for row in rows if row["method"] == METHODS[1]]
    if not individual:
        return lines + ["", "Este dataset no contiene invocaciones individuales. No se inventan p50/p95/p99.", ""]
    lines += ["", "### Invocaciones — percentil_por_invocacion", "",
              "p50 y rangos p95/p99 en µs. Nearest-rank; solo p50 recibe el umbral < 10 %.",
              "Las colas muestran rangos observados entre repeticiones y pueden variar; no son intervalos de confianza.", ""]
    lines += table(["Benchmark", "p50 R1", "p50 R2", "p50 R3", "Var. p50", "p95 min–max", "p99 min–max", "N / warmup", "Estado editorial", "Avisos / procedencia"], [
        ["`" + row["benchmark"] + "`", *[us(row["statistics_by_run"].get(run, {}).get("p50")) for run in RUNS],
         percent(row.get("relative_variation_p50")), interval(row.get("tail_ranges_ns", {}).get("p95")),
         interval(row.get("tail_ranges_ns", {}).get("p99")), str(row["descriptor"].get("n")) + " / " + str(row["descriptor"].get("warmup")),
         editorial(row), ", ".join(flag.replace("-", " ") for flag in row["instrument_flags"]) + " [artefacto](" + row["provenance"]["artifact_url"] + ")"]
        for row in individual])
    return lines + [""]


def render_cases(registry):
    lines = ["# Escenarios y fronteras temporales", "",
             "Las salidas son **esperadas por contrato del fixture**. Los recolectores consumen los resultados, pero no archivaron esos valores devueltos.",
             "Comparar estos escenarios muestra costes de operaciones distintas; no aísla la sobrecarga de WASI o IR v0.6.", ""]
    lines += table(["Benchmark", "Familia / etapa", "Operación", "Entradas", "Salida esperada (no observada)", "API Criterion / individual"], [
        ["`" + name + "`", value["family"] + " / " + value["stage"], value["operation"], compact(value["arguments"]),
         compact(value["expected_outputs_contract"]), str(value["criterion_api"]) + " / " + str(value["individual_api"])]
        for name, value in registry["scenarios"].items()])
    lines += ["", "## Qué incluye cada duración", "",
              "Cold-run: Engine nuevo + carga del artefacto + una invocación. No es arranque de proceso ni cold start de Lambda.",
              "Warm-run: Engine y artefacto precargados; Store nuevo por invocación. Cache-hit: lookup Resolver + invocación, también con Store nuevo.",
              "Criterion mide el closure completo, incluida preparación dentro de él y destrucción del resultado. La compilación IR→WASM queda fuera.",
              "El recolector individual prepara argumentos y buffers WASI antes del reloj y destruye resultados después. Esos límites no son intercambiables con Criterion.",
              "El control identity vía invoke devuelve valores; los controles invoke_with_limits devuelven InvocationReport. El segundo control sin fuel conserva la misma API y forma de retorno que el caso fuel.",
              "Los escenarios se ejecutan en orden fijo. Las diferencias de fuel pueden incluir deriva temporal; un intervalo observado no demuestra causalidad.", "",
              "Las definiciones están fijadas a hashes de fuentes por OS en cases.json. Diferencias de finales de línea entre checkouts no se confunden con equivalencia de fuentes.", ""]
    return "\n".join(lines)


def render_history(publication):
    lines = ["# Ejecuciones históricas independientes", "",
             "Datos técnicos sin certificar. Cada bloque conserva tres repeticiones propias; no se agrupan seis o nueve medianas.",
             "Mismo modelo de CPU no demuestra misma máquina física. Solo el último workflow tiene invocaciones individuales.", ""]
    datasets = publication["datasets"]
    for dataset in datasets:
        if dataset["role"] == "primary":
            continue
        lines += ["## " + dataset["id"], "", "[Workflow](" + dataset["workflow_url"] + ") · [Artefacto](" + dataset["artifact_url"] + ") · [Evidencia archivada](" + dataset["archive"] + ")"]
        lines += context(dataset["environment"])
        lines += measurement_tables([row for row in publication["rows"] if row["provenance"]["dataset_id"] == dataset["id"]])
    lines += ["## Diagnóstico macOS", ""]
    for dataset in datasets:
        if dataset["os"] != "macos":
            continue
        rows = [row for row in publication["rows"] if row["provenance"]["dataset_id"] == dataset["id"] and row["method"] == METHODS[0]]
        faster = sum(row["medians"]["run-03"] < row["medians"]["run-01"] for row in rows)
        lines += ["- " + dataset["id"] + ": " + status_counts(rows) + "; R3 menor que R1 en " + str(faster) + "/" + str(len(rows)) + " medianas."]
    lines += ["", "La deriva se repite. Los metadatos no contienen telemetría de carga, energía/frecuencia o scheduling del host: no permiten atribuir la causa a la virtualización.",
              "Mantener el 10 %; medir condiciones del host y contrastar en un runner controlado antes de relajar metodología o certificar macOS.",
              "AN-054 añadió otro instrumento y cambió la secuencia de recolección: no atribuir cambios numéricos a una mejora del runtime.", "",
              "## Comparabilidad de las dos ejecuciones AN-053", "",
              "Coincidir en modelo de CPU o imagen no identifica una misma máquina física; no se agrupan muestras.", ""]
    lines += table(["OS", "Anterior", "Verificación", "Fuentes registradas idénticas", "Mismo modelo CPU", "Misma imagen"], [
        [item["os"], item["before"], item["after"], "sí" if item["same_recorded_sources"] else "no",
         "sí" if item["same_cpu_model"] else "no", "sí" if item["same_image"] else "no"]
        for item in publication["historical_comparisons"]])
    lines += [""]
    return "\n".join(lines)


def render_readme(publication):
    lines = ["# Línea base medida candidata — pendiente de revisión humana", "",
             "## Limitaciones y estado", "",
             "**Ninguna fila está certificada.** Un workflow verde prueba recolección completa, no estabilidad ni certificación.",
             "Las filas inestables permanecen visibles y quedan excluidas de afirmaciones de línea base certificada. Las aceptables con avisos requieren revisión del instrumento.", ""]
    lines += ["- " + value for value in publication["limitations"]]
    lines += ["", "Umbral conservado: `(max - min) / mediana` de tres medianas Criterion < 10 %; por separado, la misma regla sobre tres p50 individuales.",
              "Porcentajes y µs se redondean para mostrar; la clasificación usa los valores completos. p95/p99 no reciben ese umbral inicialmente.", "",
              "## Regeneración offline", "", "```text", "python scripts/bench_publish.py", "python scripts/bench_publish.py --check", "```", "",
              "inputs.json fija las nueve fuentes y la ejecución principal; no selecciona repeticiones favorables. evidence/ conserva JSON crudo comprimido y hashes, sin depender de la caducidad de artefactos GitHub.",
              "Cada .json.gz contiene un objeto files con las rutas originales (run-XX/criterion/... y run-XX/latency/...). El compresor usa cabecera sin fecha ni nombre de máquina. Los hashes verifican integridad, no certificación ni firma de autoría.",
              "El comando verifica hashes, procedencia, fuentes de fixtures y ambos resúmenes contra las muestras antes de generar esta publicación. No necesita red ni volver a medir.",
              "Comprueba cada mediana Criterion como mediana de tiempos/iteraciones y recalcula los percentiles individuales con nearest-rank. Los intervalos bootstrap originales Criterion se conservan como evidencia; no se vuelven a simular ni se usan como intervalos de fuel.",
              "[publication.json](publication.json) contiene todas las filas con procedencia completa; [cases.md](cases.md) detalla operación, entradas, salidas esperadas y API; [history.md](history.md) conserva grupos históricos independientes.", "",
              "## Estado de las mediciones principales", ""]
    primary = [data for data in publication["datasets"] if data["role"] == "primary"]
    counts = []
    for data in sorted(primary, key=lambda item: list(OS_NAMES).index(item["os"])):
        rows = [row for row in publication["rows"] if row["provenance"]["dataset_id"] == data["id"]]
        criterion = [row for row in rows if row["method"] == METHODS[0]]
        individual = [row for row in rows if row["method"] == METHODS[1]]
        counts.append([OS_NAMES[data["os"]], status_counts(criterion), status_counts(individual), sum(bool(row["instrument_flags"]) for row in rows)])
    lines += table(["OS", "Criterion (14 filas)", "p50 individual (9 filas)", "Filas con avisos"], counts)
    for data in sorted(primary, key=lambda item: list(OS_NAMES).index(item["os"])):
        identifier = data["id"]
        rows = [row for row in publication["rows"] if row["provenance"]["dataset_id"] == identifier]
        lines += ["", "## " + OS_NAMES[data["os"]] + " — " + identifier, "",
                  "[Workflow](" + data["workflow_url"] + ") · [Artefacto original](" + data["artifact_url"] + ") · [Evidencia local](" + data["archive"] + ")"]
        lines += context(data["environment"])
        lines += measurement_tables(rows)
        lines += ["### Comparación de escenarios por invocación", "",
                  "WASM básico, WASI, fuel e IR v0.6 realizan operaciones distintas. Los costes siguientes son p50 por escenario, no sobrecargas aisladas.", ""]
        individual = [row for row in rows if row["method"] == METHODS[1]]
        lines += table(["Familia / operación", "Entradas → contrato de salida", "API", "p50 R1/R2/R3 µs", "Estado"], [
            [row["case"]["family"] + " / " + row["case"]["operation"], compact(row["case"]["arguments"]) + " → " + compact(row["case"]["expected_outputs_contract"]),
             row["descriptor"]["api"], "/".join(us(row["statistics_by_run"][run]["p50"]) for run in RUNS), editorial(row)] for row in individual])
        lines += ["", "### Instrumento", "",
                  "Sin sustracción de overhead. Salto observado = cota empírica superior de resolución intrínseca; mediana vacía = aproximación inferior del coste del instrumento.", ""]
        lines += table(["Repetición", "Salto observado ns", "Mediana vacía ns"], [
            [run, values["resolution_ns"], values["overhead_p50_ns"]] for run, values in publication["calibration"][identifier].items()])
        lines += ["", "Una mediana vacía de 0 ns puede estar por debajo de la resolución: no demuestra coste cero. Las distribuciones y muestras completas antes/después están archivadas.", "",
                  "### Diferencias de fuel", "",
                  "ns por repetición; intervalo = rango de tres diferencias emparejadas de p50. **No es un intervalo de confianza**. Puede incluir deriva temporal; no hay atribución causal.", ""]
        lines += table(["Control", "R1/R2/R3 ns", "Rango ns", "Ambos p50 estables", "Interpretación"], [
            [item["control"], "/".join(str(item.get("differences_by_run", {}).get(run, "sin evaluación")) for run in RUNS),
             compact(item.get("range_ns")), "sí" if item.get("both_p50_stable", False) else "no",
             "misma operación y API; cambia fuel" if item["control"].endswith("limits_unmetered") else "misma operación; cambian fuel, API y forma de retorno"]
            for item in publication["fuel_comparisons"][identifier]])
    lines += ["", "## Referencia para INIT-08-03", "",
              "Esta publicación permite localizar etapas y escenarios candidatos, pero no decide todavía caché o warm pools ni fija una mejora Z%.",
              "Antes de optimizar: revisar filas concretas aceptables y su instrumento, estabilizar mediciones relevantes (especialmente macOS), contrastar controles de igual operación/API y medir memoria/aislamiento donde corresponda.",
              "Cold-run vs warm-run cambia Engine/carga; no demuestra por sí solo el beneficio marginal de un warm pool. El Resolver existente ya cachea artefactos, no instancias.",
              "La certificación y el cierre de AN-055 / INIT-08-02 requieren revisión humana explícita. Las cifras Wasmtime 27/--quick de AN-017 se conservan como historial y no son la referencia para nuevas decisiones.", ""]
    return "\n".join(lines)


def outputs(inputs_path):
    publication, registry = build(inputs_path)
    return {"README.md": render_readme(publication), "cases.md": render_cases(registry),
            "history.md": render_history(publication),
            "publication.json": json.dumps(publication, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False) + "\n"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inputs", type=Path, default=DEFAULT_INPUTS)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    destination = args.output or args.inputs.parent
    rendered = outputs(args.inputs)
    stale = []
    for name, content in rendered.items():
        path = destination / name
        if args.check:
            if not path.is_file() or path.read_text(encoding="utf-8") != content:
                stale.append(name)
        else:
            destination.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content.encode("utf-8"))
    if stale:
        raise SystemExit("Outdated benchmark publication: " + ", ".join(stale))
    print("Verified offline candidate publication" if args.check else "Generated candidate publication; human certification pending")


if __name__ == "__main__":
    main()
