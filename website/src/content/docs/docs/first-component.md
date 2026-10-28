---
title: Tu primer componente
description: Verifica, compila y ejecuta una función identity que devuelve 42, sin LLM ni credenciales.
---

Este ejemplo empieza directamente con Semantic IR para aislar el recorrido de
verificación y ejecución. **No usa generación por IA.** Necesitas la
[CLI instalada](/docs/installation/) y un directorio vacío para los archivos del tutorial.

## 1. Crea la entrada

Guarda este bloque completo como `identity.json` en el directorio de trabajo.
Es una función sin imports que devuelve su único argumento entero.

```json title="identity.json"
{
  "ir_version": "0.1",
  "spec_version": 1,
  "functions": [
    {
      "name": "identity",
      "export": true,
      "params": [{ "name": "value", "type": "i32" }],
      "returns": "i32",
      "requirements": ["SMOKE-001"],
      "body": { "op": "param", "name": "value" }
    }
  ]
}
```

La versión del fixture reproduce el contrato mínimo de `cli_e2e.rs`. No representa
una promesa de estabilidad de todas las formas futuras de Semantic IR.

## 2. Verifica y compila

Ejecuta desde el mismo directorio. Estos comandos funcionan sin continuaciones de
línea tanto en PowerShell como en shells Unix.

```sh
wasma verify identity.json
wasma build identity.json --output identity.wasm
```

Ambos deben terminar con código de salida `0`; el segundo crea `identity.wasm`.
`verify` comprueba la entrada semántica. No verifica firmas de binarios.

## 3. Ejecuta

```sh
wasma run --component identity.wasm --function identity --fuel 100000 42
```

El resultado esperado es el entero **42** y código de salida `0`. El presupuesto de
fuel limita el trabajo guest; su valor aquí es un presupuesto, no una medición publicada.

## 4. Prueba el contrato

Guarda esta suite como `identity.yaml`:

```yaml title="identity.yaml"
suite: identity-tutorial
spec_version: 1
cases:
  - id: identity-42
    requirement: SMOKE-001
    function: identity
    args: [42]
    expected: 42
```

```sh
wasma test --suite identity.yaml --component identity.wasm
```

La suite debe pasar con código `0`. Para observar un error de contrato, solicita una
exportación que no existe:

```sh
wasma run --component identity.wasm --function missing 42
```

Este último comando debe fallar con código `1`. Un error explícito es preferible a
un resultado silencioso por defecto. No significa que todo error sea de dominio:
argumentos inválidos o archivos ausentes pueden terminar con código `2`.

## Qué acabas de comprobar

Entrada → verificación → compilación → invocación → aceptación. No hubo LLM en el
recorrido. Para avanzar desde una narrativa consulta los [modos de CLI](/docs/cli/),
y antes de ejecutar artefactos ajenos revisa [seguridad](/docs/security/).
