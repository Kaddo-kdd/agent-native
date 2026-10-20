---
title: Seguridad
description: Permisos explícitos, fuel, memoria e integridad antes de ejecutar un componente.
---

## Permisos explícitos

El acceso host está sujeto a políticas. Una interfaz importada no obtiene autorización
solo por aparecer en el componente: el modelo combina lo solicitado y lo concedido.
Imports no autorizados deben fallar antes de ejecutar el guest.

La garantía es concreta al alcance implementado, no una certificación de seguridad de
cualquier artefacto externo. El ejemplo `identity` no necesita imports host.

## Fuel

`--fuel` limita trabajo abstracto ejecutado por la VM. No expresa segundos ni milisegundos.
El ejemplo usa `100000` como presupuesto; no se publica como consumo exacto.
Agotar el presupuesto produce un error, no una salida de éxito parcial.

## Memoria

`--max-memory-bytes` limita memoria lineal guest y `--max-table-elements` limita tablas.
**No equivalen a un límite de RSS de todo el proceso host.** La compilación, el motor
y otros recursos host tienen costes separados.

Omitir cuotas es útil para desarrollo, pero no constituye una configuración recomendada
para código no confiable. La política de despliegue debe elegir presupuestos adecuados
y observar los errores de agotamiento.

## Integridad y procedencia

`run --verify` admite el hash esperado o un manifiesto, y verificación de firma Minisign
con clave pública según las opciones de la CLI. Obtén hash y clave por un canal confiable.
Un hash copiado del mismo artefacto no autentica su origen; una firma válida tampoco
demuestra que su comportamiento sea seguro.

`verify identity.json` verifica el IR. Es una operación distinta de verificar un
binario firmado antes de ejecutarlo.

## Aislamiento

El Store efímero por invocación evita conservar estado guest entre llamadas.
Consulta [runtime](/docs/runtime/) y no extrapoles esta regla a garantías de sistema
operativo, procesos host o servicios externos no cubiertos.

## Evidencia del proyecto

Las pruebas de [límites](https://github.com/Kaddo-kdd/agent-native/blob/main/crates/cli/tests/run_limits.rs)
y [verificación](https://github.com/Kaddo-kdd/agent-native/blob/main/crates/cli/tests/run_verify.rs)
documentan casos concretos. Revisa la versión del checkout antes de aplicar sus contratos.
