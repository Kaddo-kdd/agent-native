---
title: Qué es Wasma
description: De especificaciones funcionales a componentes WebAssembly verificables y ejecutables.
---

Wasma conecta **especificación, verificación y ejecución**. Una narrativa puede convertirse
en `SystemSpec`, después en Semantic IR, y finalmente en un componente WebAssembly.
El runtime ejecuta componentes sin depender de un LLM ni del compilador.

El proyecto está en desarrollo activo. **Wasma es el nombre público; la CLI todavía se
llama `agent-native`**. Los crates, variables de entorno y repositorio conservan ese nombre.

## Tu primer recorrido

1. [Instala la CLI](/docs/installation/) desde el repositorio.
2. [Construye tu primer componente](/docs/first-component/) con un ejemplo offline.
3. Comprende los [conceptos](/docs/concepts/) y las [fronteras de seguridad](/docs/security/).

## Dos momentos diferentes

**Construcción:** el código de la herramienta interpreta especificaciones, genera o
lee IR, verifica contratos y compila. El modo LLM real requiere un proveedor externo.

**Ejecución:** el runtime carga el artefacto, aplica las opciones de verificación y
recursos seleccionadas, e invoca una exportación. No necesita llamar a un modelo.

La verificación estructural no sustituye las pruebas de comportamiento ni garantiza
que una especificación refleje correctamente la intención humana.

## Alcance actual

- Cinco subcomandos: `build`, `verify`, `test`, `run` y `publish`.
- Políticas de permisos, aislamiento entre invocaciones y límites configurables.
- Semantic IR v0.6 experimental con `--experimental`.
- Baseline WASI con I/O controlada en memoria, sin prometer compatibilidad universal.

Consulta el [roadmap](/roadmap/) para distinguir hitos y ampliaciones pendientes.

## Fuentes y contribuciones

El código vive en [GitHub](https://github.com/Kaddo-kdd/agent-native). Las issues,
releases y la [guía de contribución](https://github.com/Kaddo-kdd/agent-native#contribuir)
son los canales del proyecto. No necesitas una cuenta en este sitio para leer o probar.
