# Decisión de cierre limitado — AN-055 e INIT-08-02

## Decisión humana aprobada

Se aprueba cerrar AN-055 como publicación reproducible de evidencia técnica con
certificación parcial limitada. INIT-08-02 mantiene su alcance cross-OS original
y permanece abierta. La decisión no cambia certificaciones ni umbrales.

## Estado de AN-055

AN-055 puede cerrarse como **publicación reproducible de evidencia técnica con
certificación parcial limitada**. La publicación ya satisface sus entregables:

| Criterio | Evidencia | Evaluación |
|---|---|---|
| AC-01 | Filas inestables visibles; siete Criterion no WASI de Windows aprobadas con estabilidad en dos workflows | Cumplido solo para esas siete referencias; no añade certificación a otras filas. |
| AC-02 | `estimacion_agregada` y `percentil_por_invocacion` se publican separados | Cumplido. |
| AC-03 y AC-07 | `python scripts/bench_publish.py` regenera y `--check` verifica sin red | Cumplido en CI. |
| AC-04 | `inputs.json`, evidencia comprimida, hashes, entorno y artefactos por fila | Cumplido. |
| AC-05 | Limitaciones al inicio de `docs/benchmarks/README.md` | Cumplido. |
| AC-06 | Cierre de INIT-08-02 exige revisión humana | Sigue pendiente; no bloquea el cierre documental de AN-055. |

La publicación conserva los p50/p95/p99 como evidencia técnica. Los avisos del
reloj de Windows impiden certificar percentiles individuales; no impiden publicar
sus datos y rangos. WASI sigue sin certificar después de AN-061 y AN-062.

### Decisión aplicada para AN-055

Se cierra AN-055 como entrega de publicación reproducible, con estas afirmaciones
máximas: siete referencias Criterion no WASI de Windows tienen certificación
humana limitada; Linux y macOS, WASI y percentiles individuales son evidencia
publicada sin certificación. Los avisos del reloj y la reproducibilidad de
rendimiento entre hosts se trasladan a investigación futura cuando afecten una
optimización o un compromiso de rendimiento concreto.

## Estado de INIT-08-02

La iniciativa sigue abierta bajo su definición actual de **línea base pública
cross-OS**. ADR-007 designa Linux x86_64 GNU como plataforma operativa
recomendada, pero no convierte sus mediciones en una línea base de rendimiento
certificada: las dos sesiones disponibles cambiaron de CPU registrada y AN-062
observó variabilidad en Linux.

| Alcance | Estado actual |
|---|---|
| Publicación trazable cross-OS | Disponible, con evidencia técnica por los tres sistemas. |
| Compatibilidad funcional | Linux, Windows y macOS validados por CI. |
| Referencia de rendimiento limitada | Siete operaciones Criterion no WASI de Windows. |
| Plataforma operativa recomendada | Linux x86_64 GNU, por ADR-007; no es certificación de rendimiento. |
| WASI y percentiles individuales | Sin certificación. |
| Línea base cross-OS certificada | No disponible. |

### Opciones de decisión para INIT-08-02

1. **Mantenerla abierta.** **Opción aprobada.** Conserva el alcance cross-OS
   original hasta disponer de condiciones comparables para las afirmaciones de
   rendimiento necesarias.
2. **Cerrar con alcance redefinido.** Requiere aprobar explícitamente que el
   objetivo pasa de línea base cross-OS certificada a publicación técnica
   cross-OS, una referencia operativa Linux sin cifras certificadas y siete
   referencias Criterion limitadas de Windows. No autoriza comparar sistemas,
   certificar WASI ni percentiles, ni fijar una mejora Z%.

La primera opción preserva literalmente los criterios originales. La segunda
permite iniciar INIT-08-03, pero cada experimento de caché o warm pool deberá
usar una comparación controlada en Linux x86_64 GNU: misma máquina o runner
controlado, mismo componente, entrada, API y fronteras temporales. El umbral de
mejora se establecerá a partir de ese control, no de diferencias cross-OS.

## Límites que no cambian

- Umbral estricto de 10 % para la estabilidad local de Criterion y p50.
- Siete referencias Criterion no WASI de Windows certificadas parcialmente.
- Windows x86_64 y macOS ARM64 compatibles con validación funcional CI.
- WASI, p50/p95/p99 y comparaciones de rendimiento entre sistemas sin
  certificación.
- AN-061 y AN-062 permanecen completadas como diagnósticos; su cierre no
  resuelve la estabilidad del runtime.

## Recomendación

Se aprobó el cierre limitado de AN-055 y se mantiene INIT-08-02 abierta hasta
una decisión posterior sobre su alcance. Esto separa una entrega documental
completada de la decisión de producto necesaria para iniciar caché y warm pool
sin presentar una línea base universal inexistente.
