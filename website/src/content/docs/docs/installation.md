---
title: Instalación
description: Compila la CLI agent-native desde fuente y comprueba sus cinco comandos reales.
---

## Requisitos

Necesitas Git, Rust y Cargo. El repositorio fija Rust **1.85.0** mediante
`rust-toolchain.toml`. Con rustup instalado, Cargo selecciona ese toolchain al entrar
al checkout. En Windows usa las herramientas de compilación MSVC; en Linux/macOS,
el linker y herramientas nativas correspondientes.

Node y pnpm solo son necesarios para desarrollar este sitio, no para ejecutar la CLI.

## Compilar desde fuente

```sh
git clone https://github.com/Kaddo-kdd/agent-native.git
cd agent-native
cargo build --locked --release -p agent-native
```

El binario queda en `target/release/agent-native` (o `agent-native.exe` en Windows).
Puedes añadir ese directorio a tu PATH o invocarlo con la ruta completa.

```sh
./target/release/agent-native --help
./target/release/agent-native --version
git rev-parse HEAD
```

En PowerShell puedes usar `./target/release/agent-native.exe`. Registra el SHA de Git
además de `--version`: la versión del workspace puede no identificar por sí sola el hito.

## Comprobar la instalación

La ayuda debe listar `build`, `verify`, `test`, `run` y `publish`. No existe un
subcomando `spec` en el parser usado por esta documentación.

Los siguientes ejemplos asumen que `agent-native` está en el PATH. Si no lo está,
sustituye ese prefijo por la ruta absoluta al binario compilado.

## Releases y artefactos

Consulta las [releases del repositorio](https://github.com/Kaddo-kdd/agent-native/releases)
para conocer el estado publicado. No asumas que cada tag incluye un binario descargable:
al revisar `v0.0.7-m7` no había assets adjuntos. Los artefactos de CI pueden requerir
acceso a GitHub y expirar; por eso este tutorial utiliza instalación desde fuente.

La evidencia de ejecución de esta web se mantiene en
[website/validation](https://github.com/Kaddo-kdd/agent-native/tree/main/website/validation),
con SHA, plataforma y procedencia del ejecutable empleado.

## Continúa

[Ejecuta tu primer componente](/docs/first-component/). No requiere claves LLM ni
servicios de red una vez compilada la CLI.
