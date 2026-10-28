---
type: adr
id: ADR-008
status: accepted
date: 2026-10-28
related_work_items:
  - AN-077
  - AN-078
  - AN-079
code:
  - Cargo.toml
  - crates/cli/Cargo.toml
  - crates/ir/Cargo.toml
  - crates/compiler/Cargo.toml
  - crates/runtime/Cargo.toml
---

# ADR-008 — Consolidación de Identidad Técnica y Estrategia de Migración a Wasma

## Context

El proyecto completó su núcleo técnico (INI-001 a INI-008, 67 Work Items) bajo el nombre de desarrollo inicial `agent-native`. No obstante, la marca comercial y el portal público operan bajo la identidad **Wasma** (`wasma.org`).

Antes de la primera release pública **v0.1.0** (INI-009), existía una discrepancia entre la marca pública y las interfaces de software:
- El ejecutable principal se compilaba como `agent-native`.
- El paquete Cargo de la CLI se llamaba `agent-native`.
- Los crates de biblioteca se llamaban `agent-native-ir`, `agent-native-compiler` y `agent-native-runtime`.
- El repositorio GitHub remoto está en `https://github.com/Kaddo-kdd/agent-native`.
- Las variables de entorno para configuración LLM usan el prefijo `AGENT_NATIVE_LLM_*`.

La evaluación realizada en AN-077 identificó una **colisión en crates.io**: el nombre de paquete `wasma` ya está registrado por un proyecto no relacionado (*Windows Assignment System Monitoring Architecture* de Azencorporation). Sin embargo, los nombres `wasma-cli`, `wasma-ir`, `wasma-compiler` y `wasma-runtime` se encuentran completamente disponibles. En GitHub, el repositorio `Kaddo-kdd/wasma` se encuentra disponible.

## Decision

Se aprueba formalmente la adopción de la identidad **Wasma** mediante una **migración técnica integral faseada**, con las siguientes decisiones de diseño:

### 1. Nombres de Paquetes y Crates en Cargo
- **CLI:** El paquete Cargo se denominará **`wasma-cli`**, produciendo exclusivamente el binario ejecutable **`wasma`** (`[[bin]] name = "wasma"`). Esto resuelve la colisión en `crates.io` y garantiza que la experiencia del usuario final sea `wasma <comando>`.
- **Librerías:** Los crates se renombrarán a **`wasma-ir`** (modelo/verificador IR), **`wasma-compiler`** (compilador) y **`wasma-runtime`** (motor de ejecución).
- **Dependencias internas:** El workspace actual no utiliza `[workspace.dependencies]`; cada crate declara dependencias relativas (`path = "../ir"`). En AN-078 se actualizarán los nombres locales relativos. Los requisitos de `path + version` para publicación en crates.io se abordarán en AN-068/AN-070.

### 2. Política del Ejecutable Anterior y Alias
- La instalación estándar de `wasma-cli` **proporcionará únicamente el ejecutable `wasma`**.
- La compatibilidad con scripts existentes que invoquen `agent-native` se implementará como una opción explícita y **deshabilitada por defecto** mediante una feature flag de Cargo (`--features legacy-alias`).
- **Política de deprecación:** Dicho alias imprimirá en `stderr` una advertencia indicando que el comando está obsoleto y será retirado formalmente en Wasma v0.2.0.

### 3. Variables de Entorno y Manejo Seguro
- Se aprueba el prefijo **`WASMA_LLM_*`** (`WASMA_LLM_KEY`, `WASMA_LLM_PROVIDER`, `WASMA_LLM_MODEL`, `WASMA_LLM_ENDPOINT`, `WASMA_LLM_TIMEOUT`) como configuración canónica.
- **Precedencia:** `WASMA_LLM_*` prevalece estrictamente sobre `AGENT_NATIVE_LLM_*`.
- **Valores vacíos:** Cadenas vacías o con solo espacios en blanco se tratan como no definidas, activando el fallback o valor por defecto.
- **Protección de credenciales:** Si se recurre al fallback de `AGENT_NATIVE_LLM_*`, la CLI emitirá en `stderr` un warning mencionando exclusivamente el nombre de la variable, sin exponer jamás su valor o secreto.

### 4. Disponibilidad y Registro Externo
- Se deja constancia de que `wasma-cli` resuelve la colisión en `crates.io`, pero no garantiza la exclusividad del comando `wasma` a nivel de sistema operativo (PATH).
- Se establece como requisito re-verificar la disponibilidad de los paquetes en `crates.io` inmediatamente antes de su publicación efectiva en AN-076.

### 5. Repositorio GitHub, Integraciones y Rollback Seguro
- Se aprueba la migración de `Kaddo-kdd/agent-native` a **`Kaddo-kdd/wasma`**, sujeta a autorización humana explícita en AN-079.
- No se asume que las redirecciones de GitHub cubran todas las herramientas; AN-079 auditará GitHub Actions, la integración de Netlify para `wasma.org`, badges, webhooks y automatizaciones.
- Se descarta `git reset --hard` como estrategia de recuperación. Se adopta una política de aislamiento en ramas de trabajo y reversión mediante commits trazables (`git revert`), preservando modificaciones locales y el historial de Git.

### 6. Garantías Arquitectónicas Invariables
- Se mantienen estrictamente intactos los contratos de Semantic IR v0.5/v0.6, WIT, Canonical ABI, políticas de seguridad y la garantía de Store efímero e instancia fresca (ADR-005).
- Los Work Items completados (`AN-001` a `AN-067`) y los ADRs previos (`ADR-001` a `ADR-007`) permanecen inmutables como registro histórico.

### 7. Secuencia de Trabajo
- **AN-077:** Evaluación técnica, inventario y aprobación de este ADR (Spike, aceptado).
- **AN-078:** Implementación de la migración en el workspace Rust, código, tests, CI y portal web (Feature).
- **AN-079:** Renombramiento del repositorio GitHub y verificación de integraciones (Chore).

## Consequences

### Positivas
- Identidad técnica coherente en CLI, documentación, crates y repositorio.
- Cero colisiones en `crates.io` manteniendo el ejecutable público como `wasma`.
- Transición segura y controlada sin exponer credenciales ni contaminar sistemas de usuarios con binarios legados por defecto.
- Trazabilidad y gobernanza de Kaddo preservadas.

### Negativas / Costes
- Requiere actualizar ~192 imports en Rust y 4 manifiestos de crates en AN-078.
- Actualización obligatoria de comandos de build en CI y scripts que invoquen `-p agent-native` (pasando a `-p wasma-cli`).
- Requiere re-configuración de remotos locales tras AN-079.

## Validation

1. Compilación limpia del workspace con `cargo check --workspace` y `cargo test --workspace` bajo la identidad Wasma.
2. Invocación de `wasma --help` y `wasma --version` confirmando el comando oficial.
3. Ejecución de benchmarks reales existentes: `cargo test --locked -p wasma-runtime --example bench_latency`.
4. Validación del portal con `pnpm --dir website test` y `website/scripts/test-docs.mjs` evaluando comandos `wasma`.
5. Verificación de CI multiplataforma en Linux, Windows y macOS.
