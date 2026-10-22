# Workers experimentales — campaña A/A (AN-067)

La campaña A/A de workers se ejecuta manualmente en Linux x86_64 GNU. Compara
el camino secuencial de `Tenant::invoke` con `experimental-workers` usando el
mismo componente, presupuesto de fuel, warmup y carga ofrecida.

```text
python scripts/worker_bench_collect.py --output worker-bench-results \
  --workers 1,2 --queue-capacities 1,4 --concurrency 1,2,4 \
  --repetitions 3 --warmup 1000 --requests 10000
```

Cada directorio de campaña contiene un manifiesto, muestras por solicitud,
resúmenes recalculables, señales RSS disponibles y `SHA256SUMS`. Los resultados
no se mezclan con Criterion ni certifican rendimiento universal. Una campaña
prolongada define su duración antes de comenzar mediante `--soak-seconds`.

En Linux, el recolector toma muestras RSS cada 50 ms del proceso de la sesión y
sus descendientes, junto con el offset relativo y el máximo recalculable. La
serie permite comparar tendencias durante un soak; no prueba la presencia o
ausencia de fugas. El intervalo puede omitir picos breves y la suma de procesos
puede contar páginas compartidas más de una vez. En plataformas no POSIX, el
manifiesto declara la señal RSS como no disponible.