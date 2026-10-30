# Changelog

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/)
y este proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

> [!NOTE]
> Las etiquetas y commits previos en el repositorio de Git corresponden a puntos de control y validaciones del desarrollo técnico interno (Iniciativas INI-001 a INI-008). No representan distribuciones públicas oficiales previas. Wasma v0.1.0 constituye la primera versión candidata oficial para distribución pública.

---

## [0.1.0] - En preparación

Esta versión candidata v0.1.0 consolida el desarrollo técnico completo del motor Wasma (anteriormente Agent Native), formalizando su identidad, compilador, runtime standalone y CLI unificada bajo el estándar WebAssembly Component Model.

### Añadido

#### Representación Intermedia Semántica (`wasma-ir`)
- Modelo de datos fuertemente tipado para agentes deterministas.
- Soporte normativo para la especificación Semantic IR v0.5 (tipos escalares, registros, listas, importaciones y exportaciones).
- Soporte experimental opt-in para la especificación Semantic IR v0.6 (tipos de cadena y resultado extendidos, y primitivas declarativas de estado, desacopladas del runtime).
- Validador formal de esquemas JSON y verificación de integridad estructural.

#### Compilador de Componentes WebAssembly (`wasma-compiler`)
- Transformación directa de Semantic IR a binarios estándar del WebAssembly Component Model.
- Síntesis y empaquetado de interfaces WIT canónicas (`wasi:cli@0.2.0`, `wasi:clocks@0.2.0`, `wasi:filesystem@0.2.0`, `wasi:io@0.2.0`, `wasi:random@0.2.0`).
- Generación de artefactos reproducibles y deterministas con cálculo de hashes SHA-256.

#### Runtime Standalone (`wasma-runtime`)
- Motor de ejecución independiente basado en Wasmtime 29 y WASI Preview 2.
- Aislamiento seguro por componente sin dependencias de WASI Preview 1 legado.
- Límites de recursos configurables (memoria, gas/instrucciones, tiempo de CPU y límites de invocación).
- Integración de host determinista para HTTP y llamadas seguras a proveedores de inferencia LLM.
- Soporte arquitectónico para workers host-side acotados con aislamiento de memoria (deshabilitados por defecto bajo el flag `experimental-workers`).

#### Interfaz de Línea de Comandos (`wasma-cli` / `wasma`)
- Binario unificado `wasma` con los cinco comandos centrales del ciclo de vida:
  - `build`: Compilación de especificaciones IR a componentes WASM.
  - `verify`: Verificación estática y validación de esquemas, hashes y firmas.
  - `test`: Ejecución de pruebas unitarias y de integración sobre componentes compilados.
  - `run`: Ejecución directa del componente en el runtime standalone con paso de argumentos y configuración.
  - `publish`: Empaquetado atómico de artefactos firmados con manifiesto y trazabilidad.
- Soporte de firma y verificación criptográfica de artefactos mediante Minisign.
- Variables de entorno oficiales `WASMA_LLM_*` con fallback transparente a variables heredadas.
- Alias de compatibilidad hacia atrás `agent-native` (opcional, deshabilitado por defecto mediante feature flag `legacy-alias`).

#### Infraestructura y Plataformas
- Suite de pruebas de regresión y benchmarks cross-OS con certificación en Linux x86_64, Windows x86_64 y macOS (ARM64/x86_64).
- Integración continua en GitHub Actions (`Kaddo-kdd/wasma`) y portal de documentación `wasma.org`.

### Cambiado
- Consolidación de la identidad técnica del proyecto de Agent Native a Wasma (ADR-008).
- Renombramiento unificado de los paquetes Cargo a `wasma-cli`, `wasma-ir`, `wasma-compiler` y `wasma-runtime`.
- Actualización de metadatos de repositorio canónico a `https://github.com/Kaddo-kdd/wasma`.

### Limitaciones Conocidas
- **Workers concurrentes:** La funcionalidad de workers en el host permanece en fase experimental (`experimental-workers`) y deshabilitada por defecto en compilaciones estándar.
- **WASI:** Soporte exclusivo para WASI Preview 2 (0.2.0). Componentes basados en `wasi_snapshot_preview1` no son compatibles.
- **Empaquetado crates.io:** Los manifests de los crates declaran `version = "0.1.0"` y rutas relativas; su empaquetado formal para crates.io requiere publicación secuencial en orden de dependencias.
