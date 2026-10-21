---
id: AN-055
type: docs
status: in-progress
created_at: 2026-10-20T00:00:00.000Z
title: Publicación reproducible de la línea base de rendimiento en docs/benchmarks
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - docs/benchmarks/
  - scripts/bench_publish.py
  - scripts/bench_evidence.py
  - scripts/test_bench_publish.py
  - .github/workflows/ci.yml
  - knowledge/tech/baselines/runtime-baseline.md
source_work_item_candidate:
  - WI-CANDIDATE-08-02-03
depends_on:
  - AN-053
  - AN-054
source_roadmap_initiative: INIT-08-02
initiative: INI-008
domains:
  - Performance
  - Runtime & Memory
related_capabilities:
  - Independent Runtime Execution
expected_value: >-
  Línea base publicada, trazable y reproducible de GitHub-hosted runners, sin
  afirmar más de lo que los datos respaldan.
---

# AN-055 — Publicación de la línea base

## Alcance

1. Reporte en `docs/benchmarks/` generado a partir de los artefactos de AN-053 y
   AN-054, con un comando de regeneración documentado.
2. Cada fila incluye: commit, fecha de medición, sistema operativo, modelo de CPU,
   versión de toolchain y Wasmtime, fixture, unidad, número de muestras, resultado de
   la regla de variación y enlace a los artefactos originales.
3. Sección de **definiciones** (cold-run, warm-run, cache-hit) y de **limitaciones**:
   runners de GitHub con hardware variable, no es un benchmark universal por sistema
   operativo, no compara con implementaciones nativas ni con AWS Lambda.
4. Los benches `inestable` o `hardware-mixto` conservan sus cifras en los resultados
   técnicos con su motivo, excluidos de afirmaciones de línea base certificada.
5. Actualizar `runtime-baseline.md` (AN-017) indicando que su cifra de Wasmtime 27 y
   `--quick` queda reemplazada por esta línea base, conservando el historial.
6. Tabla de coste por invocación: WASM básico, WASI, fuel e IR v0.6. Identificar
   etapa, operación, entradas/salidas, API y método. Distinguir comparación de
   escenarios de estimación de sobrecarga: diferencias entre operaciones distintas
   no prueban el coste aislado de una característica. Para fuel, comparar la misma
   operación y explicitar cualquier diferencia de API y retorno.
7. Separar la estabilidad de medianas Criterion y p50 individual. Publicar p95/p99
   con rangos entre las tres corridas y advertencias, sin umbral inicial del 10 %.
   Documentar resolución y overhead calibrados, sin restarlos automáticamente.
8. Distinguir repetibilidad local (tres corridas en un job) de reproducibilidad
   (workflow posterior independiente). Mantener sus datos separados y enlazados.
   La tabla revisada será referencia de INIT-08-03 para decidir si caché o warm
   pools justifican su complejidad mediante mejoras medidas.

## Criterios de aceptación

- AC-01: todo valor certificado procede de corridas completas (no `--quick`) con
  estabilidad aceptable por método; los valores excluidos siguen visibles como
  resultados técnicos. No aplicar el umbral del 10 % a p95/p99 inicialmente.
- AC-02: percentiles etiquetados `percentil_por_invocacion`; Criterion etiquetado
  `estimacion_agregada`.
- AC-03: el reporte se regenera con un único comando a partir de los artefactos.
- AC-04: cada fila es trazable a commit, entorno, fixture, unidad, N, fecha y artefacto.
- AC-05: las limitaciones están al inicio del documento.
- AC-06: INIT-08-02 solo pasa a `completed` tras revisión humana de la publicación.

## Condición de cierre

Requiere aprobación explícita antes de declarar la línea base como certificada.
Mantener este work item en `draft` en el commit documental. Esperar mediciones
reales antes de publicar cifras; INIT-08-02 no pasa a `completed` con este commit.

## Refinamiento y materialización (2026-10-20)

AN-053 y AN-054 están integrados y sus artefactos medidos y validados. Se autoriza
implementar esta publicación, con estado explícito **candidata, sin certificar**.
La implementación puede terminar antes de la revisión humana; el cierre de este
work item y de INIT-08-02 sigue condicionado a esa revisión.

### Entregables concretos

1. `docs/benchmarks/inputs.json`: inventario fijo con workflow, artefacto, commit
   medido, OS, papel (principal/histórico) y hashes de la evidencia archivada.
   Principal: run 39449438474 (AN-054, ambos métodos). Históricos independientes:
   39441524333 y 39446048498 (AN-053, solo Criterion). Nunca seleccionar la mejor
   repetición ni agrupar muestras entre workflows u OS.
2. Evidencia JSON comprimida determinista por dataset bajo `docs/benchmarks/evidence/`:
   entorno, estado de recolección, los tres JSON crudos Criterion por benchmark,
   muestras individuales y calibración cuando existan, y resúmenes originales.
   Conservar las entradas necesarias para regenerar, sin versionar gráficos y logs
   derivados. Los enlaces originales permanecen como procedencia adicional.
3. Un único comando offline, `python scripts/bench_publish.py`, valida hashes,
   reconstruye los artefactos y recalcula ambos resúmenes desde las muestras antes
   de generar Markdown y JSON. `--check` comprueba que la publicación está al día.
   Fallar ante alteraciones, datos parciales o procedencia inconsistente.
4. Tablas principales separadas por OS y método: 14 filas Criterion y 9 individuales.
   Cada fila JSON contiene commit, fecha UTC, CPU, OS/arquitectura, toolchain,
   Wasmtime, fixture, método, N, unidad y referencia al artefacto y archivo crudo.
   Markdown enlaza esos datos sin repetir toda la ficha de entorno en cada celda.
5. Tabla de escenarios con operación, entradas, salidas **esperadas por contrato**,
   API y frontera temporal. Los instrumentos no archivaron los valores devueltos:
   no presentar esas salidas como observaciones medidas. Comparar identity, WASI
   echo y v0.6 describe operaciones; no calcular sobrecargas causales entre ellas.
6. Mostrar las diferencias de fuel por repetición y su rango observado, separando
   el control `invoke` del control de igual API sin fuel. No es un intervalo de
   confianza; la deriva temporal y los avisos impiden atribuciones causales.
7. Historial separado de ejecuciones y diagnóstico macOS (14→13 filas inestables
   en los dos runs AN-053, sin mezclar sus medianas). No afirmar reproducibilidad
   individual: solo hay una ejecución independiente del recolector AN-054.
8. Aviso de sustitución de referencia en AN-017, conservando íntegro el snapshot
   de Wasmtime 27/--quick. La referencia actual es candidata, no certificada.
9. Validación offline en CI de integridad, regeneración y límites metodológicos.

### Clasificación editorial

- `inestable`, hardware mixto o evidencia incompleta: visible técnicamente y
  excluida de afirmaciones de línea base certificada.
- `aceptable` con avisos de instrumento: requiere revisión del instrumento;
  superar el umbral de repetibilidad no elimina esos avisos.
- `aceptable` sin avisos: candidata a revisión humana, nunca certificada por código.
- p95/p99 muestran sus rangos y advertencias; no reciben el umbral del 10 %.

### Criterios adicionales de aceptación

- AC-07: la publicación se regenera sin red con los archivos versionados y comprueba
  los hashes antes de leer datos; CI detecta evidencia alterada o tablas desactualizadas.
- AC-08: el inventario de filas principal contiene 69 filas (3 OS × (14 + 9)),
  sin eliminar las inestables; históricos separados y procedencia completa en JSON.
- AC-09: ninguna cifra se etiqueta certificada; la referencia para INIT-08-03
  explicita qué información falta antes de decidir caché o warm pools.

## Implementación disponible para revisión

- `docs/benchmarks/README.md`, `cases.md`, `history.md` y `publication.json`
  generados por `scripts/bench_publish.py`, sin acceso a red.
- 9 archivos de evidencia (aprox. 1,5 MB comprimidos), 153 filas técnicas en total:
  69 principales y 84 históricas. Todas las filas permanecen sin certificar.
- Matriz principal: 31 candidatas sin avisos, 5 aceptables con revisión de
  instrumento pendiente y 33 inestables excluidas de afirmaciones certificadas.
  Esta clasificación editorial no constituye aprobación humana.
- Integridad, procedencia, inputs incompletos/quick, contratos de escenarios y
  regeneración comprobados con 36 pruebas Python; verificación offline en CI
  para Linux, Windows y macOS.
- AN-017 conserva sus cifras históricas y remite a la nueva candidata. INIT-08-02
  sigue abierto; no se fija Z% ni se decide caché/warm pools antes de revisar datos.

## Dictamen humano de publicación (2026-10-20)

Se acepta el informe como **publicación técnica candidata**. La evidencia, el
mecanismo de regeneración y la trazabilidad quedan validados como base para una
revisión posterior, incluida su comprobación offline en CI.

No se certifica automáticamente ninguna de las 31 filas `aceptable` sin avisos.
Tampoco se certifican las cinco filas con avisos de instrumento ni se declara una
línea base cross-OS completamente certificada. Todas las filas continúan con su
clasificación técnica actual y sin reclamar una certificación que los datos no
sostienen.

### Evidencia adicional requerida antes del cierre

1. Ejecutar AN-054 nuevamente desde `main` como workflow independiente y
   conservar sus tres repeticiones como un dataset nuevo, sin mezclar muestras
   ni resúmenes con las ejecuciones anteriores.
2. Comparar las variaciones entre sesiones, con revisión específica de macOS,
   y documentar si el runner Apple M1 virtualizado condiciona la referencia.
3. Caracterizar o resolver los avisos de resolución del reloj en Windows antes
   de certificar percentiles por invocación.
4. Proponer una matriz explícita que identifique, por operación y entorno,
   qué filas pueden aprobarse como referencias limitadas, cuáles siguen como
   evidencia técnica y qué limitaciones acompañan cada afirmación.

La estabilidad no es requisito para conservar evidencia técnica, ni se exige que
todos los benchmarks sean estables para proponer una línea base parcial. AN-055
e INIT-08-02 permanecen `in-progress` hasta que la matriz de certificación parcial
y la nueva ejecución independiente reciban aprobación humana explícita.

### Verificación independiente de AN-054 (run 39534970668)

La ejecución manual desde `main` sobre el commit
`679490606a398e807c19dd6dfa2dec794de6311d` concluyó correctamente en Linux,
macOS y Windows. Sus tres artefactos se archivaron como datasets de verificación
independientes en `docs/benchmarks/evidence/39534970668/`; no sustituyen la
fuente primaria ni mezclan muestras con ella.

- Linux: 10/14 filas Criterion y 6/9 p50 individuales fueron `aceptable`.
- Windows: 11/14 filas Criterion y 5/9 p50 fueron `aceptable`; los cinco p50
  aceptables mantienen el aviso de resolución del reloj.
- macOS: 1/14 filas Criterion y 1/9 p50 fueron `aceptable`; la variación sigue
  impidiendo una referencia certificable en ese runner.

La regeneración offline comprobó hashes, procedencia, muestras y los resúmenes
recalculados de los doce datasets. Esta sesión aporta repetibilidad entre
workflows, pero no certifica filas ni permite fijar todavía un objetivo Z%.

La propuesta de certificación parcial está en
`docs/benchmarks/certification-matrix.md`: delimita siete referencias Criterion
no WASI de Windows para decisión humana y mantiene Linux, macOS y los percentiles
individuales como evidencia técnica.

### Dictamen técnico sobre la matriz (2026-10-21)

Las siete referencias Criterion no WASI de Windows son favorables para una
aprobación humana limitada al entorno registrado. `cold_run_component_model_wasi_echo`
permanece pendiente de una nueva confirmación con runtime idéntico o de una
equivalencia WASI documentada y aprobada. p50, p95 y p99 no se certifican; Linux
y macOS permanecen como evidencia técnica. Este dictamen no sustituye la
aprobación humana explícita ni cierra AN-055 o INIT-08-02.

Como seguimiento metodológico, el recolector debe registrar una huella de las
fuentes efectivas del runtime además de las fuentes de escenarios. Esa mejora
evitará incertidumbre de equivalencia en futuras certificaciones.

### Aprobación humana parcial registrada (2026-10-21)

El responsable del proyecto aprobó explícitamente siete referencias `Criterion`
no WASI de Windows: `engine_construct`, `load_component_model_point_sum`,
`cold_run_core_identity`, `cold_run_component_model_point_sum`,
`warm_run_core_identity`, `warm_run_component_model_v06_greet` y
`warm_run_component_model_v06_safe_div`. La aprobación se materializa en
`docs/benchmarks/inputs.json` y se valida al regenerar: exige los datasets
primario `39449438474-windows` y de verificación `39534970668-windows`, fuentes,
CPU, imagen, Rust y Wasmtime coincidentes, y estado `aceptable` en ambos workflows.

Es una referencia parcial limitada al runner y versiones archivadas; no certifica
p50/p95/p99, WASI, Linux, macOS ni una línea base cross-OS. AN-055 sigue abierto:
queda confirmar WASI en dos sesiones con una huella efectiva del runtime y resolver
las limitaciones de instrumento antes de cualquier certificación adicional.
