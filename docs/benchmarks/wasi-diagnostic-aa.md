# Diagnóstico A/A de variabilidad WASI — AN-062

## Alcance y procedencia

Este diagnóstico conserva dos sesiones independientes de `wasi-diagnostic`, sin
mezclar sus muestras. Ambas ejecutaron el commit
`e7ff5a97c9f87cd8ac9ed138dbe863dd6131a452`, con tres repeticiones Criterion
por etapa y por sistema operativo. Las tres etapas son observaciones separadas:
construcción de `Engine`, carga de componente con política e invocación WASI
sobre un componente previamente cargado. Sus tiempos no se suman ni sustituyen
el benchmark histórico `cold_run_component_model_wasi_echo`.

| Sesión | Workflow | Estado | Evidencia archivada |
|---|---|---|---|
| 1 | [39561303285](https://github.com/Kaddo-kdd/agent-native/actions/runs/39561303285) | éxito, 3/3 | [Linux](evidence/39561303285/linux.json.gz), [Windows](evidence/39561303285/windows.json.gz), [macOS](evidence/39561303285/macos.json.gz) |
| 2 | [39563138519](https://github.com/Kaddo-kdd/agent-native/actions/runs/39563138519) | éxito, 3/3 | [Linux](evidence/39563138519/linux.json.gz), [Windows](evidence/39563138519/windows.json.gz), [macOS](evidence/39563138519/macos.json.gz) |

Cada archivo comprimido contiene 35 JSON permitidos: ficha de entorno, estado
de colección y las muestras/estimaciones Criterion de las tres etapas para las
tres repeticiones. `scripts/bench_evidence.py` verificó cada archivo contra sus
muestras antes de archivarlo.

## Resultados dentro de cada sesión

Medianas Criterion en µs. La variación es `(máximo - mínimo) / mediana de las
tres medianas`; `aceptable` exige un valor estrictamente menor que 10 %.

| OS | Sesión | Engine: R1/R2/R3 · variación · estado | Carga con política: R1/R2/R3 · variación · estado | Invocación WASI: R1/R2/R3 · variación · estado |
|---|---:|---|---|---|
| Linux | 1 | 0.943/0.925/0.956 · 3.27 % · aceptable | 1108.478/1044.360/1069.478 · 6.00 % · aceptable | 33.493/36.463/37.649 · 11.40 % · inestable |
| Linux | 2 | 1.631/1.579/1.591 · 3.30 % · aceptable | 2010.336/2013.078/2037.197 · 1.33 % · aceptable | 92.989/92.553/86.150 · 7.39 % · aceptable |
| Windows | 1 | 1.643/1.670/1.657 · 1.68 % · aceptable | 1209.068/1210.707/1241.197 · 2.65 % · aceptable | 32.740/33.281/35.378 · 7.93 % · aceptable |
| Windows | 2 | 2.935/2.618/2.642 · 12.02 % · inestable | 2288.552/2268.943/2288.336 · 0.86 % · aceptable | 60.164/60.487/58.600 · 3.14 % · aceptable |
| macOS | 1 | 1.703/1.595/1.432 · 16.99 % · inestable | 1252.238/1159.671/1169.924 · 7.91 % · aceptable | 33.491/29.892/32.188 · 11.18 % · inestable |
| macOS | 2 | 1.950/1.937/2.089 · 7.78 % · aceptable | 1872.791/1911.435/1840.845 · 3.77 % · aceptable | 50.250/58.627/45.159 · 26.80 % · inestable |

La carga con política fue aceptable en las seis observaciones. Engine fue
aceptable en cuatro de seis; la invocación WASI, en tres de seis. Estos conteos
no constituyen una tasa de fiabilidad ni una certificación de rendimiento.

## Comparación entre sesiones y entorno registrado

Las huellas efectivas del runtime fueron iguales dentro de cada OS entre ambas
sesiones: `ac3d…790d` para Linux/macOS y `0e88…f291` para Windows. Las imágenes
registradas también fueron iguales por OS. Sin embargo, Linux y Windows
recibieron modelos de CPU distintos entre sesiones (`EPYC 9V45` en la sesión 1;
`EPYC 7763` en la sesión 2), por lo que sus cambios absolutos de medianas no son
comparaciones A/A sobre el mismo host físico. macOS registró `Apple M1 (Virtual)`
en ambas, sin prueba de identidad física.

| OS | Afinidad / CPUs disponibles | Reloj observado | Cambio de carga durante las repeticiones | Lectura |
|---|---|---|---|---|
| Linux | `0-3` / 4 en ambas sesiones | 20–30 ns sesión 1; 40 ns sesión 2 | baja de 4.52 a 2.34 y de 4.45 a 1.96 | Afinidad constante; no se expuso frecuencia. |
| Windows | `F` / 4 en ambas sesiones | 100 ns en ambas | sesión 1: 6→3, 9→12, 3→1; sesión 2: 2→0, 8→4, 0→0 | Afinidad y frecuencia estables dentro de cada sesión; frecuencia registrada cambió 2596→2445 MHz entre sesiones. |
| macOS | no disponible / 3 en ambas | 41 ns en ambas | baja de 44.99 a 18.48 y de 40.59 a 20.61 | No se expusieron afinidad ni frecuencia; la carga es observacional, no causal. |

La telemetría elimina cambios registrados de afinidad y reloj dentro de cada
sesión como explicación suficiente, pero no mide frecuencia efectiva,
co-residencia, planificación del hipervisor ni identidad de máquina. Por ello
no atribuye la variación a WASI, Wasmtime, Criterion ni al runner.

## Dictamen técnico

- La instrumentación separa el comportamiento observado: la carga con política
  no mostró inestabilidad local en estas seis observaciones.
- La inestabilidad de Engine es intermitente en Windows y macOS. La de invocación
  WASI se repite en macOS, es mixta en Linux y no apareció dentro de las dos
  sesiones de Windows.
- La hipótesis de una diferencia de fuentes del runtime queda debilitada para
  cada OS, porque las huellas son iguales entre sesiones. No queda descartado el
  efecto del entorno alojado: Linux y Windows cambiaron de modelo de CPU entre
  sesiones, y ningún runner prueba identidad de host físico.
- WASI continúa sin certificar. Estas mediciones diagnósticas no alteran las
  siete referencias Criterion limitadas de Windows, AN-061, AN-055 ni INIT-08-02.

No se recomienda una tercera sesión idéntica antes de decidir un experimento
que controle el host o capture señales adicionales de planificación/frecuencia.
AN-062 permanece abierto mientras la interpretación y su siguiente experimento
reciben revisión humana.

## Integridad

| Archivo | SHA-256 comprimido |
|---|---|
| `39561303285/linux.json.gz` | `4aa4978dd088fb4f22c3b41d634a5caeeb904090bee14f620695ef5eeb6363b9` |
| `39561303285/windows.json.gz` | `60533cb2c013e290f003f18835f8469a679124d58e52e24666535ea4da14eb8a` |
| `39561303285/macos.json.gz` | `17824e9003b8ad7c18d4c52b511e12a33ff9057b0a0f351e567a9e1928e5bcf7` |
| `39563138519/linux.json.gz` | `a1a67e46700383a3f811dd189712256206dedc87ec5cc0bd18cd81470508aad9` |
| `39563138519/windows.json.gz` | `52d3d0c6a3f008e13ea434c41b6bc00dd6b1f5a113bc068e798f0a7550c7c426` |
| `39563138519/macos.json.gz` | `d6b46dcf01ed6e02ba88f8a3386e9abea5ac0aa32965ed0315c489dd85909031` |
