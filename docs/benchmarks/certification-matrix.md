# Matriz de certificación parcial — aprobación humana registrada

## Alcance de la revisión

Esta matriz compara dos workflows completos, sin agrupar sus muestras: la fuente
primaria AN-054 `39449438474` y la verificación independiente
`39534970668`. Ambos contienen tres repeticiones por sistema operativo. Los
hashes de las fuentes de escenarios, Rust, Wasmtime y el modo de medición
coinciden en cada par; CPU e imagen del runner se evalúan por separado.

La aprobación humana del 2026-10-21 certifica únicamente siete referencias
Criterion no WASI de Windows. No autoriza comparaciones entre sistemas
operativos, pooling de muestras, SLA ni un objetivo Z% para INIT-08-03.

| OS | Criterion aceptable en ambos workflows | Entorno registrado coincidente | Resolución |
|---|---:|---|---|
| Linux | 7/14 | No: CPU e imagen difieren | Mantener como evidencia técnica; no proponer referencia entre sesiones. |
| Windows | 8/14 | Sí: CPU, imagen, Rust, Wasmtime y fuentes coinciden | Siete no WASI certificadas parcialmente; WASI requiere confirmación adicional. |
| macOS | 0/14 | Sí, pero la variación persiste | Mantener como evidencia técnica; no proponer referencia. |

## Referencias Criterion no WASI certificadas parcialmente para Windows

Estas siete operaciones no WASI fueron `aceptable` dentro de cada workflow y se midieron
con el mismo perfil registrado de Windows. La certificación humana se limita al
runner GitHub-hosted `win25-vs2026 / 20261007.264.1`, CPU `AMD EPYC 7763 64-Core
Processor`, Rust 1.85.0, Wasmtime 29.0.1 y los commits archivados. No representa
una línea base universal de Windows ni una garantía para otra máquina física.

| Operación | Dataset primario | Verificación independiente | Estado |
|---|---|---|---|
| `engine_construct` | [39449438474-windows](evidence/39449438474/windows.json.gz) | [39534970668-windows](evidence/39534970668/windows.json.gz) | Certificada parcialmente el 2026-10-21 |
| `load_component_model_point_sum` | 39449438474-windows | 39534970668-windows | Certificada parcialmente el 2026-10-21 |
| `cold_run_core_identity` | 39449438474-windows | 39534970668-windows | Certificada parcialmente el 2026-10-21 |
| `cold_run_component_model_point_sum` | 39449438474-windows | 39534970668-windows | Certificada parcialmente el 2026-10-21 |
| `warm_run_core_identity` | 39449438474-windows | 39534970668-windows | Certificada parcialmente el 2026-10-21 |
| `warm_run_component_model_v06_greet` | 39449438474-windows | 39534970668-windows | Certificada parcialmente el 2026-10-21 |
| `warm_run_component_model_v06_safe_div` | 39449438474-windows | 39534970668-windows | Certificada parcialmente el 2026-10-21 |

## Referencia WASI pendiente

`cold_run_component_model_wasi_echo` fue aceptable en ambos workflows, pero no
se certifica todavía. La evidencia archivada no registra una huella de las fuentes
efectivas del runtime; por tanto, no permite demostrar que el cambio WASI entre
sesiones sea equivalente para fines de certificación. Se requiere repetir el
benchmark con una versión idéntica del runtime en dos sesiones independientes o
documentar y aprobar explícitamente esa equivalencia.

## Percentiles individuales

No se certifica ningún p50 individual todavía.

- Linux conserva tres p50 aceptables en ambos workflows, pero cambiaron CPU e
  imagen del runner entre sesiones. No hay una referencia intersesión delimitada.
- Los cuatro p50 aceptables en ambos workflows de Windows tienen el aviso
  `mediana-de-overhead-no-resuelta-por-reloj`.
- macOS solo conserva `warm_run_core_identity_fuel` como p50 aceptable en ambos
  workflows; el resto de la sesión mantiene variación alta.

Los p95 y p99 siguen siendo descriptivos: publican rangos entre repeticiones y
no reciben una regla de certificación en esta fase.

## Trabajo pendiente

1. Autorizar la confirmación adicional para `cold_run_component_model_wasi_echo`.
2. Mantener Linux, macOS y todos los percentiles individuales como evidencia
   técnica hasta obtener condiciones comparables o resolver sus avisos.
3. Mantener AN-055 e INIT-08-02 abiertos. Esta matriz no fija Z% ni autoriza la
   implementación de caché o warm pools.
