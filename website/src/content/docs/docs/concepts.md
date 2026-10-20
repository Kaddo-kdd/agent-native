---
title: Conceptos principales
description: SystemSpec, Semantic IR, WIT, WebAssembly y Component Model en el recorrido de Wasma.
---

## SystemSpec

Representación estructurada de una especificación funcional. Organiza componentes,
contratos y requisitos para conservar contexto durante construcción y evolución.
La interpretación de una narrativa y la generación de IR son pasos distintos.

## Semantic IR

Representación intermedia de funciones, tipos, operaciones y requisitos. El verifier
comprueba la estructura y compatibilidad soportadas antes de compilar. No es Rust ni
JavaScript generado. Su repertorio de operaciones es acotado.

IR v0.6 incorpora capacidades experimentales, como `string` y `result<T, E>`, bajo
`--experimental`. No asumas soporte de bucles o primitivas futuras por analogía con
un lenguaje de programación general.

## WIT

WebAssembly Interface Types describe interfaces e intercambios de valores entre
componentes. Una signatura expresa un contrato, no concede automáticamente permisos.
Los imports deben estar soportados y autorizados por la política del runtime.

## WebAssembly y Component Model

WebAssembly es el formato de ejecución; Component Model añade composición e interfaces
tipadas. Wasma trabaja con un alcance específico de artefactos Core y Component Model.
No promete ejecutar cualquier módulo producido por cualquier lenguaje.

## Verificación, pruebas y ejecución

- La **verificación** revisa el IR o la especificación.
- La **compilación** genera el artefacto.
- Las **pruebas de aceptación** comprueban comportamientos definidos.
- El **runtime** invoca funciones aplicando límites y permisos.

Estas capas se complementan. La validez estructural no demuestra que el comportamiento
resuelva el problema del usuario, y una firma no demuestra seguridad del código.

## Generación y determinismo

`fixtures` carga casos predefinidos; `synthesizer` aplica reglas heurísticas; `real`
consulta un proveedor LLM. La salida generativa puede variar. El determinismo de la
compilación o del ejemplo offline no equivale a prometer narrativa libre → mismo
programa bajo cualquier proveedor o modelo.

[Ver el recorrido práctico](/docs/first-component/) · [Consultar el runtime](/docs/runtime/)
