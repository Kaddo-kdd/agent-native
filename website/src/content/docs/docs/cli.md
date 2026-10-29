---
title: Referencia CLI
description: Los cinco subcomandos reales de wasma, sus opciones y los modos de generación.
---

La referencia se contrasta con
[`crates/cli/src/cli.rs`](https://github.com/Kaddo-kdd/wasma/blob/main/crates/cli/src/cli.rs).
Usa `wasma --help` y `wasma <comando> --help` para tu checkout concreto.

## Comandos

| Comando          | Entrada y propósito                 | Opciones principales                                            |
| ---------------- | ----------------------------------- | --------------------------------------------------------------- |
| `verify <input>` | Verifica `.md` o `.json`            | `--experimental`                                                |
| `build <input>`  | Compila narrativa o IR              | `--output`, `--component`, `--mode`, `--experimental`           |
| `run`            | Invoca una exportación de `.wasm`   | `--component`, `--function`, argumentos posicionales            |
| `test`           | Ejecuta aceptación YAML             | `--suite`, `--component`                                        |
| `publish`        | Publica en almacén local versionado | `--spec`, `--dist-dir`, `--source-wi`, `--git-commit`, `--mode` |

`build` recibe el archivo como argumento posicional. La opción `--spec` pertenece a
`publish`, no a `build`. No hay comandos `wasma init` ni `wasma spec`.

## Modos de construcción

| Modo          | Qué hace                                              | Qué requiere                                                  |
| ------------- | ----------------------------------------------------- | ------------------------------------------------------------- |
| `synthesizer` | Transformaciones heurísticas deterministas, limitadas | Especificación compatible; no interpreta narrativa arbitraria |
| `fixtures`    | Artefactos predefinidos para casos soportados         | Fixture reconocido                                            |
| `real`        | Generación mediante proveedor LLM y validación del IR | Configuración del proveedor y `WASMA_LLM_KEY (o AGENT_NATIVE_LLM_KEY)`          |

El adaptador real también utiliza `WASMA_LLM_ENDPOINT` y
`WASMA_LLM_MODEL` según configuración. No introduzcas claves en esta web,
capturas o archivos versionados. El runtime no requiere esas credenciales.

## Ejemplo ejecutable

El [tutorial](/docs/first-component/) proporciona los archivos de entrada completos:

```sh
wasma build identity.json --output identity.wasm
wasma run --component identity.wasm --function identity --fuel 100000 42
```

## Opciones de ejecución

`--fuel`, `--max-memory-bytes` y `--max-table-elements` configuran límites.
`--verify` habilita verificaciones previas con `--expected-sha256`, `--manifest`
o firma `--signature` y `--public-key`, según el contrato de verificación.

No confundas `verify <input>` con `run --verify`: el primero trata la entrada
semántica; el segundo comprueba integridad/procedencia del artefacto antes de invocarlo.

## Salida y errores

La opción global `--format human` es la predeterminada; `--format json` facilita
automatización. Código `0`: éxito. Código `1`: fallo de dominio. Código `2`: error
de argumentos o I/O. Comprueba el código de salida además del texto mostrado.

La publicación de CLI se realiza en un almacén local: no es un registro remoto ni
un despliegue del sitio web.
