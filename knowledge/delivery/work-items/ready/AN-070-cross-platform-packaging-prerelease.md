---
id: AN-070
type: feature
status: ready
created_at: 2026-10-26T00:00:00.000Z
title: 'Empaquetado, instalación y prerelease multiplataforma'
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-03
priority: P0
depends_on:
  - AN-069
domains:
  - Distribution & Release
code:
  - .github/workflows/**
  - docs/installation.md
  - crates/cli/tests/release_packaging_test.rs
expected_value: >-
  Generar paquetes binarios comprimidos y reproducibles para Linux x86_64 GNU,
  Windows x86_64 MSVC y macOS ARM64 con la identidad técnica aprobada de Wasma y
  smoke tests en entornos limpios.
project_state: ai-assisted
template_version: 1
ready_at: '2026-10-31'
---

# AN-070 — Empaquetado, instalación y prerelease multiplataforma

## Contexto y problema

Los usuarios y evaluadores de Wasma deben poder descargar e instalar un binario precompilado para su sistema operativo (Linux x86_64, Windows x86_64 y macOS ARM64) sin necesidad de disponer del toolchain de Rust instalado. AN-069 estableció la compilación base, el empaquetado reproducible, la limpieza de staging y la verificación estricta de inventario en CI. AN-070 formaliza la verificación de instalación y los smoke tests de ejecución autónoma en runners limpios, junto con las guías canónicas de instalación manual para cada plataforma.

## Alcance

- Verificar que los paquetes `.tar.gz` (Linux GNU y macOS ARM64) y `.zip` (Windows MSVC) contienen exactamente la estructura requerida: binario ejecutable (`wasma` o `wasma.exe`), `LICENSE` y `README.md`.
- Implementar y formalizar jobs o pasos de smoke test en runners limpios que descarguen el artefacto empaquetado de su respectiva plataforma, lo descompriman e invoquen de forma autónoma:
  - `wasma --help` (comprobando código de retorno 0 y mención a la identidad técnica Wasma).
  - `wasma --version` (comprobando `wasma 0.1.0` o versión de desarrollo esperada de acuerdo al contrato de versionamiento).
- Redactar y auditar la guía de instalación manual en [`docs/installation.md`](file:///docs/installation.md) cubriendo las 3 plataformas objetivo con rutas estándar (`/usr/local/bin`, `%USERPROFILE%\bin` o variables `PATH`).
- Asegurar que la ejecución de los binarios no dependa de variables de entorno heredadas ni de dependencias dinámicas ajenas a los sistemas operativos base soportados.

## Criterios de aceptación

- AC-01: El pipeline compila y empaqueta binarios reproducibles en modo release bajo la identidad técnica aprobada para las 3 plataformas objetivo (`x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` y `aarch64-apple-darwin`), excluyendo directorios de staging.
- AC-02: Cada paquete comprimido contiene exclusivamente el binario ejecutable (`wasma` / `wasma.exe`) con permisos correctos (0755 en Unix), junto con `LICENSE` y `README.md`.
- AC-03: Pruebas automatizadas de desempaquetado y smoke tests en runners limpios (Linux, Windows, macOS) ejecutando `wasma --help` y `wasma --version` con retorno exitoso (código 0).
- AC-04: Guía de instalación manual verificada en `docs/installation.md` para cada uno de los 3 sistemas operativos sin suposiciones de rutas heredadas de Agent Native.

## Fuera de alcance

- Instalador binario desatendido tipo script `curl | sh` (evaluable tras validar seguridad y firma).
- Publicación formal de releases en GitHub (reservado para AN-076).
- Empaquetado para gestores de paquetes de terceros (Homebrew, winget, APT/RPM).

## Validación

1. Verificación de archivos comprimidos: Comprobar la estructura interna de los archivos `.tar.gz` y `.zip` y el contenido de `LICENSE` y `README.md`.
2. Verificación en CI de smoke tests: Ejecución exitosa de desempaquetado y llamada a `wasma --help` y `wasma --version` en runners limpios de Linux, Windows y macOS.
3. Verificación de permisos y ejecutabilidad: Confirmar que el binario extraído posee flags de ejecución en entornos POSIX sin requerir `chmod +x` manual por parte del usuario.
4. Verificación documental: Auditoría de `docs/installation.md` para exactitud de rutas, variables y compatibilidad multiplataforma.

## Entregable

Paquetes multiplataforma validados con smoke tests automatizados en entornos limpios y documentación de instalación canónica para usuarios finales.
