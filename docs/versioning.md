# Contrato y política de versionamiento de Wasma

Este documento define la política formal de versionamiento para el proyecto Wasma a partir de la versión candidata **v0.1.0**, estableciendo la relación y desacoplamiento entre los crates del workspace Rust, la especificación Semantic IR, los contratos de interfaces WIT del Component Model y el portal público.

---

## 1. Crates del Workspace Rust

El workspace Rust de Wasma está compuesto por cuatro paquetes principales:
- `wasma-ir`: Modelo de datos, esquema y validador de la Representación Intermedia Semántica.
- `wasma-compiler`: Compilador AOT/JIT de IR a componentes WebAssembly estándar (WASM Component Model).
- `wasma-runtime`: Motor de ejecución standalone basado en Wasmtime con soporte WASI Preview 2.
- `wasma-cli`: Interfaz de línea de comandos unificada (`wasma`, con subcomandos `build`, `verify`, `test`, `run`, `publish`).

### 1.1. Reglas SemVer 2.0.0
Los cuatro crates comparten una política unificada de versiones (*lockstep versioning*) gobernada por `[workspace.package] version = "0.1.0"`.

- **Fase inicial pre-1.0 (`0.y.z`):**
  - `0.y.0` (cambio en `MINOR`): Indica cambios que introducen rupturas de compatibilidad de API pública, cambios en la semántica del compilador o runtime, o modificaciones en los comandos de la CLI.
  - `0.y.z` (cambio en `PATCH`): Indica adición de características compatibles hacia atrás, mejoras de rendimiento o correcciones de errores (bug fixes).
- **Fase post-1.0 (`x.y.z`):**
  - Cumplimiento estricto de SemVer 2.0.0: `MAJOR` para cambios incompatibles, `MINOR` para funcionalidad compatible, `PATCH` para correcciones compatibles.

### 1.2. Dependencias internas y orden de publicación
Para garantizar tanto la compilación local dentro del repositorio como la viabilidad de empaquetado para distribución en crates.io (cuando sea aprobada institucionalmente), todas las dependencias internas del workspace combinan la ruta local y la versión mínima requerida:

```toml
wasma-ir = { path = "../ir", version = "0.1.0" }
wasma-compiler = { path = "../compiler", version = "0.1.0" }
wasma-runtime = { path = "../runtime", version = "0.1.0" }
```

**Orden topológico de publicación en crates.io:**
1. `wasma-ir`
2. `wasma-compiler` (depende de `wasma-ir`)
3. `wasma-runtime` (depende de `wasma-ir`; dev-dependency en `wasma-compiler`)
4. `wasma-cli` (depende de `wasma-ir`, `wasma-compiler`, `wasma-runtime`)

*Nota sobre empaquetado:* `cargo package` verifica por defecto la presencia de las dependencias en el registro remoto crates.io. Durante la preparación local previa a la publicación inicial, la verificación del workspace se realiza mediante `cargo check --workspace` y `cargo test --workspace`. La publicación real de crates está reservada para decisiones posteriores a AN-076.

---

## 2. Política independiente de Semantic IR

La versión de la especificación Semantic IR es **independiente** de la versión de los crates de Rust y sigue su propio ciclo de evolución formal:

- **Semantic IR v0.5 (Estable / Normativa):**
  - Es el contrato base del proyecto y el formato estándar generado y verificado.
  - Define tipos escalares (`string`, `i32`, `i64`, `f32`, `f64`, `bool`), estructuras (`record`), listas (`list`), importaciones puras y exportaciones deterministas.
  - Garantiza compatibilidad bidireccional y verificación formal contra esquemas JSON.
- **Semantic IR v0.6 (Experimental):**
  - Especificación de datos y esquemas en evolución que introduce capacidades de modelo declarativo extendido (tipos compuestos adicionales como cadenas y resultados en funciones puras, y primitivas declarativas de estado).
  - La compatibilidad con v0.6 está estrictamente aislada en el compilador/verificador y requiere habilitación mediante flags de verificación.
  - El crate `wasma-ir@0.1.0` es capaz de parsear y validar tanto la especificación estable v0.5 como la sintaxis experimental v0.6.

### 2.1. Desacoplamiento explícito entre Semantic IR v0.6 y Workers del Runtime
Es crucial diferenciar las extensiones de la especificación Semantic IR v0.6 de la funcionalidad de workers concurrentes del runtime:
- **Semantic IR v0.6** es un contrato de **representación de datos y tipos** que rige cómo se expresan y validan las especificaciones antes de su compilación.
- **Workers host-side** es una capacidad de **ejecución en el motor de host** (`wasma-runtime`), gobernada por la feature de Cargo `experimental-workers`, centrada en aislamiento de memoria, pools de instancias y límites de concurrencia.
- Ambas características son completamente independientes y no forman parte del mismo contrato de versionamiento ni comparten ciclo de vida: la especificación IR no asume la presencia de workers en el host, y el soporte de workers en el runtime opera sobre componentes WebAssembly estándar independientemente de si la especificación fuente fue v0.5 o v0.6.

---

## 3. Contratos de interfaces WIT (WebAssembly Component Model)

Las interfaces externas de interoperabilidad se definen mediante WebAssembly Interface Type (WIT) y se anclan a especificaciones estándar del Bytecode Alliance:

- **WASI Preview 2 (0.2.0):**
  - `wasi:cli@0.2.0`: Soporte de entrada/salida estándar, variables de entorno y argumentos de línea de comandos.
  - `wasi:clocks@0.2.0`: Monotonic y wall-clock timers.
  - `wasi:filesystem@0.2.0`: Acceso virtualizado y pre-abierto al sistema de archivos.
  - `wasi:io@0.2.0`: Flujos de lectura/escritura deterministas.
  - `wasi:random@0.2.0`: Generación segura de entropía.
- **Inmutabilidad de contratos:** Los componentes WASM generados por `wasma-compiler` importan interfaces WIT canónicas. El runtime `wasma-runtime` confina las implementaciones de host a estas interfaces exactas, denegando llamadas heredadas a WASI Preview 1 (`wasi_snapshot_preview1`).

---

## 4. Portal Web (`wasma.org`)

El sitio web y documentación (`website/`) está gestionado como un proyecto Astro / Starlight independiente:
- Su versión actual en `website/package.json` es `0.1.0`.
- El despliegue es continuo a través de Netlify vinculado a la rama principal de `Kaddo-kdd/wasma`.
- Los cambios en la documentación, tutoriales o diseño visual no incrementan la versión de los paquetes Rust.
- Los lanzamientos de los crates Rust se reflejan documentalmente en el portal actualizando las guías de instalación y notas de versión.

---

## 5. Limitaciones conocidas y exclusiones para v0.1.0

La versión candidata v0.1.0 se define bajo los siguientes límites arquitectónicos:

1. **Workers host-side experimentales:**
   - La concurrencia de workers en el host está implementada pero permanece **deshabilitada por defecto** bajo el feature flag `experimental-workers` en `wasma-runtime`. No se recomienda su uso en producción hasta completar validaciones de aislamiento adicionales.
2. **Exclusividad WASI Preview 2:**
   - Wasma no soporta binarios WASI Preview 1. Todos los componentes deben cumplir con el estándar Wasmtime Component Model (Preview 2).
3. **Ejecutable y alias:**
   - El ejecutable principal y oficial es `wasma`.
   - El alias de compatibilidad `agent-native` se encuentra deshabilitado por defecto y solo está disponible mediante compilación con `--features legacy-alias`.
4. **Entornos soportados (Tier-1):**
   - Linux x86_64 (glibc / musl).
   - macOS aarch64 (Apple Silicon) y macOS x86_64.
   - Windows x86_64 (MSVC).
