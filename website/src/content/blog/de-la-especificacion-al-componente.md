---
title: "Wasma: de la especificación al componente"
description: "La identidad pública de Agent Native y una idea central: separar la intención, la verificación y la ejecución."
date: 2026-10-20
tags: [Proyecto, Arquitectura]
---

Un componente ejecutable es más útil cuando podemos explicar qué debe hacer, comprobar
sus entradas y entender bajo qué condiciones se ejecuta. Esa es la dirección de Wasma.

## Una identidad para un proyecto en construcción

Wasma es el nombre público del proyecto que técnicamente sigue viviendo en
[`Kaddo-kdd/agent-native`](https://github.com/Kaddo-kdd/agent-native).
El ejecutable se llama `agent-native`; los crates y las variables de entorno conservan
su nombre. Este artículo presenta el proyecto, no anuncia una nueva release.

## De la intención a un contrato

El recorrido conecta una especificación con una representación intermedia semántica,
un verifier, un compilador y un componente WebAssembly. La generación asistida por IA
ocurre al construir. El runtime ejecuta el artefacto sin depender de un LLM.

Esta separación permite revisar cada frontera. Una salida generada no se acepta solo
porque parezca razonable: debe ajustarse a la estructura que comprende el verifier.
Eso no demuestra por sí solo que satisfaga toda intención humana; las pruebas de
aceptación siguen siendo necesarias.

## Qué se puede explorar hoy

La CLI conecta verificación, compilación, ejecución, pruebas y publicación local.
El runtime incorpora políticas, límites de recursos y verificación de integridad.
El [primer componente](/docs/first-component/) muestra un recorrido offline pequeño,
sin credenciales, con una entrada y una salida conocidas.

El modo `synthesizer` es una transformación heurística, no un intérprete libre de lenguaje
natural. El modo `real` sí requiere un proveedor LLM en tiempo de construcción. Esta
distinción importa al evaluar qué puede generar cada modo.

## Lo que sigue abierto

La evolución incluye mediciones de rendimiento y trabajo sobre concurrencia.
Los [benchmarks actuales](/docs/performance/) son evidencia candidata, no cifras
certificadas. Semantic IR v0.6 es experimental y el baseline WASI no debe confundirse
con soporte arbitrario para cualquier binario externo.

El [roadmap](/roadmap/) enlaza la evidencia de cada hito. Las decisiones, limitaciones
y contribuciones permanecen abiertas a revisión en el repositorio.

## Empieza por algo comprobable

Instala la herramienta, ejecuta el ejemplo y cambia una entrada. Después consulta los
[conceptos](/docs/concepts/) y la [seguridad](/docs/security/). Un componente pequeño y
verificable es un buen punto de partida para una conversación más ambiciosa.
