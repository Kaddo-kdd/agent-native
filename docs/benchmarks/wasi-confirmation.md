# Confirmación WASI independiente — AN-061

Dos workflows manuales independientes ejecutaron solamente
`cold_run_component_model_wasi_echo` en el mismo commit
`96c9179319ee374c84708a1f5c8ecd819a7f19e1`. Cada dataset conserva tres
repeticiones propias; no se agrupan sus muestras ni se selecciona una sesión por
su resultado.

## Trazabilidad

| Sesión | Workflow | Evidencia archivada |
|---|---|---|
| 1 | [39549485766](https://github.com/Kaddo-kdd/agent-native/actions/runs/39549485766) | `docs/benchmarks/evidence/39549485766/` |
| 2 | [39550537942](https://github.com/Kaddo-kdd/agent-native/actions/runs/39550537942) | `docs/benchmarks/evidence/39550537942/` |

Para cada OS, las dos sesiones registran el mismo commit, CPU, imagen de runner,
fuentes de escenarios y las 27 fuentes efectivas del runtime. La comparación es
por OS; una huella Windows no se trata como equivalente a una huella Linux o
macOS.

| OS | Sesión 1: R1 / R2 / R3 µs | Variación | Sesión 2: R1 / R2 / R3 µs | Variación | Conclusión |
|---|---|---:|---|---:|---|
| Linux | 2212.118 / 2211.003 / 2186.810 | 1.14 % | 2244.833 / 2239.765 / 2279.267 | 1.76 % | Estable localmente en ambas. |
| Windows | 2432.161 / 2658.245 / 2360.538 | 12.24 % | 2306.445 / 2365.680 / 2389.963 | 3.53 % | Consistencia entre sesiones no demostrada. |
| macOS | 987.910 / 1026.037 / 984.006 | 4.25 % | 1258.578 / 1082.326 / 1829.547 | 59.37 % | Consistencia entre sesiones no demostrada. |

La huella de runtime de Windows es
`caab99978a586ffeca26e422fb23ab5474e3a4b15cf81b85b43a8edb3f4ad0ec` en
ambas sesiones; sus mapas de fuentes efectivas y de escenarios son idénticos.
CPU: `AMD EPYC 7763 64-Core Processor`. Imagen:
`win25-vs2026 / 20261007.264.1`.

## Decisión metodológica

WASI permanece **sin certificar**. La segunda sesión Windows cumple el umbral
local del 10 %, pero la primera no; ambas siguen visibles. No se modifican el
umbral, las siete referencias Criterion ya certificadas ni el estado abierto de
AN-055 e INIT-08-02.

Antes de otra medición, investigar la variabilidad de Criterion, el runner y el
instrumento en Windows y macOS. Una nueva sesión sin diagnóstico no resolvería
la inconsistencia observada.
