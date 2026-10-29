---
title: Runtime
description: Ejecución independiente, ciclo de vida de las instancias y límites del baseline WASI.
---

## Una frontera independiente

El crate de runtime ejecuta artefactos sin depender del compilador ni de un proveedor
LLM. La CLI conecta las capacidades del sistema, pero no convierte al runtime en un
servicio HTTP ni exige un proceso de generación en cada invocación.

## Ciclo de vida

La arquitectura utiliza un Store efímero por invocación y descarta su estado tras la
llamada. Esta regla evita asumir reutilización segura de memoria guest entre tenants.
La reutilización de componentes compilados no equivale a reutilizar instancias vivas.

La decisión se documenta en
[ADR-005](https://github.com/Kaddo-kdd/wasma/blob/main/knowledge/tech/decisions/ADR-005-store-efimero-invariantes-aislamiento.md).

## Entradas, salidas y errores

La invocación selecciona una función exportada y argumentos compatibles. Imports no
autorizados y formas no soportadas fallan explícitamente. Las pruebas YAML pueden
comprobar resultados e I/O en memoria dentro del alcance soportado.

## WASI y consola

El baseline implementa I/O determinista en memoria con autorización granular y
límites de salida. No es una promesa de TTY interactiva, filesystem abierto o red.
La ampliación para binarios reales `wasm32-wasip2` corresponde a AN-050, que seguía
pendiente durante la preparación de esta documentación.

## Recursos

Configura [fuel y memoria](/docs/security/) antes de ejecutar componentes no confiables.
El runtime no transforma automáticamente una invocación sin límites en una política
de producción. Consulta el [roadmap](/roadmap/) para trabajo futuro de warm pool y
concurrencia; las mediciones no prueban que esas capacidades ya estén entregadas.
