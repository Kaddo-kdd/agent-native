# Decisión de cierre condicionado — AN-061 y AN-062

## Decisión humana aprobada

Se aprueba cerrar AN-061 y AN-062 como trabajos de investigación diagnóstica.
La decisión no certifica WASI ni declara estable el rendimiento del runtime.

## Excepción metodológica aceptada

AN-062 pidió un A/A en un entorno controlado o runner dedicado. Se realizaron
dos sesiones independientes con el mismo SHA, API, fixture, entrada, límites y
fronteras temporales, pero en runners hospedados de GitHub. La evidencia
registró que Linux y Windows cambiaron de modelo de CPU entre sesiones; macOS
conservó la etiqueta `Apple M1 (Virtual)` sin demostrar identidad de host físico.

Se acepta que esa condición no se cumplió en el sentido de hardware
controlado. El alcance aprobado queda limitado a identificar y documentar la
variabilidad bajo entornos registrados, no a explicarla causalmente ni a medir
reproducibilidad entre hosts.

## Evidencia que satisface el cierre diagnóstico

- AN-061 conservó dos sesiones WASI con el mismo SHA y huellas efectivas del
  runtime, descartando las fuentes registradas como explicación suficiente.
- AN-062 conservó seis artefactos A/A con tres etapas no solapadas, tres
  repeticiones por etapa y telemetría antes/después.
- La carga con política fue estable en las seis sesiones; Engine e invocación
  WASI mostraron variabilidad dependiente de etapa y sistema operativo.
- El dictamen y los hashes verificables están en
  [wasi-diagnostic-aa.md](wasi-diagnostic-aa.md).

## Límites que continúan vigentes

- WASI no queda certificado.
- AN-055 e INIT-08-02 permanecen abiertos.
- Las siete referencias Criterion limitadas de Windows permanecen sin cambios.
- El umbral estricto de 10 % no cambia.
- Una investigación futura sobre estabilidad entre hosts requerirá un runner
  dedicado o controles de entorno adicionales; queda fuera de AN-062.

## Efecto aplicado

| Elemento | Estado aplicado |
|---|---|
| AN-061 | completed |
| AN-062 | completed, con la excepción metodológica registrada |
| AN-055 | in-progress |
| INIT-08-02 | in-progress |
| Certificación WASI | pendiente |

La aprobación no autoriza nuevas cifras ni una tercera sesión. Cierra los dos
trabajos de diagnóstico y permite decidir por separado qué evidencia mínima
necesita INIT-08-02 para avanzar hacia INIT-08-03.
