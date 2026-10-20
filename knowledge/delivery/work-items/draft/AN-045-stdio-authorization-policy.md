---
id: AN-045
type: feature
status: draft
created_at: 2026-10-20T00:00:00.000Z
title: Autorización granular de stdio y streams bajo Requested ∩ Granted
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: high
code:
  - crates/runtime/src/policy.rs
  - crates/runtime/src/manifest.rs
  - crates/runtime/src/wasi.rs
  - crates/runtime/src/engine.rs
  - crates/runtime/src/tenant.rs
  - crates/runtime/tests/wasi_policy.rs
source_roadmap_initiative: INIT-07-01
source_work_item_candidate:
  - WI-CANDIDATE-07-01-02
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
expected_value: >-
  Aplicar el modelo formal de seguridad Requested ∩ Granted (ADR-004) y deny-by-default a las interfaces
  de entrada/salida de WASI 0.2 (`wasi:cli/stdin`, `wasi:cli/stdout`, `wasi:cli/stderr`, `wasi:io/streams`),
  permitiendo conceder permisos de forma granular e independiente (p. ej., autorizar stdout sin autorizar stdin,
  ni filesystem ni red) y rechazando con `RuntimeError::UnauthorizedImport` cualquier intento de acceso no autorizado
  antes de la ejecución del código guest.
risks:
  - >-
    R-SEC-WASI-01: Auto-concesión implícita de capacidades WASI. Registrar proveedores en el linker podría exponer
    por error interfaces no concedidas por la política del tenant.
    Mitigación: El linker o el validador de importaciones evalúa estrictamente `HostPolicy::is_allowed` para cada
    interfaz WIT antes de permitir la instanciación.
  - >-
    R-SEC-WASI-02: Ambigüedad entre la macro-capability `stdio` y las interfaces WIT concretas.
    Mitigación: Definir reglas canónicas de expansión: declarar `stdio` en el manifiesto autoriza explícitamente
    el trío de consola (`stdin`, `stdout`, `stderr`) y su soporte de `streams`, pero permite alternativamente
    autorizar interfaces individuales (`wasi:cli/stdout`) con máxima granularidad.
dependencies:
  - AN-033
  - AN-034
  - AN-044
source_signals:
  - 'Roadmap RM-07: INIT-07-01 capability stdio autorizada'
  - 'ADR-004: Deny-by-default en capabilities del host y fail-explicit'
  - 'Decisión de Arquitectura RM-07: Permisos separados de I/O e interfaces WIT concretas'
decision_candidates: []
related_decisions:
  - ADR-004
---

# AN-045 — Autorización granular de stdio y streams bajo Requested ∩ Granted

## Actor y Resultado
Un operador de seguridad, tenant o manifiesto declara explícitamente qué interfaces de entrada y salida de WASI 0.2 están autorizadas para un componente. El runtime evalúa la intersección estricta (Requested ∩ Granted): si el componente requiere `wasi:cli/stdout` pero la política solo concede `wasi:cli/stdin` (o ninguna), la invocación es rechazada inmediatamente con `RuntimeError::UnauthorizedImport` sin ejecutar código guest. Si la interfaz está concedida, el componente opera normalmente sobre su stream asignado.

---

## Principios y Precisiones Arquitectónicas

### 1. Descomposición Granular de Permisos
A diferencia de los runtimes convencionales que aplican flags binarios globales (`--allow-all` o `--inherit-stdio`), `agent-native` exige principio de menor privilegio:
- **`wasi:cli/stdout`:** Autoriza únicamente la emisión de salida estándar.
- **`wasi:cli/stderr`:** Autoriza la emisión de diagnósticos/errores.
- **`wasi:cli/stdin`:** Autoriza la lectura de datos de entrada.
- **`wasi:io/streams`:** Interfaz fundamental compartida para lectura/escritura en streams. Autorizar `stdout` requiere conceder implícita o explícitamente la capacidad de operar sobre el handle de output-stream correspondiente.
- **Aislamiento Total de Otras Áreas:** La concesión de permisos de consola NUNCA concede acceso a:
  - `wasi:filesystem/*` (lectura/escritura en disco permanece prohibida deny-by-default).
  - `wasi:sockets/*` (acceso a red permanece prohibido).
  - `wasi:clocks/wall-clock` (hora de reloj del sistema no autorizada salvo concesión explícita).

### 2. Modelo Declarativo en Manifiestos y `HostPolicy`
Se soportan dos modalidades en `HostPolicy` y en manifiestos YAML/JSON:
1. **Modalidad Granular Directa:**
   ```yaml
   capabilities:
     wit:
       - "wasi:cli/stdout@0.2.0"
       - "wasi:io/streams@0.2.0"
   ```
2. **Macro-Capability Conceptual (`stdio`):**
   ```yaml
   capabilities:
     categories:
       - "stdio"
   ```
   Al cargar la política, la categoría conceptual `stdio` se expande canónicamente a las interfaces:
   `wasi:cli/stdin@0.2.0`, `wasi:cli/stdout@0.2.0`, `wasi:cli/stderr@0.2.0` y `wasi:io/streams@0.2.0`.
   Si el manifiesto declara únicamente `stdout`, solo se concede `wasi:cli/stdout@0.2.0` y las operaciones de escritura asociadas de `wasi:io/streams@0.2.0`.

### 3. Mecanismo de Enforzamiento Fail-Explicit
- El validador `tenant.validate_component(&component)` y el pipeline de instanciación en `invoke.rs` inspeccionan las importaciones del Component Model.
- Si un componente importa `wasi:cli/stdin@0.2.0` y la política concedida no contiene dicha interfaz, se genera un error tipado inmediato:
  ```rust
  RuntimeError::UnauthorizedImport {
      module: "wasi:cli/stdin@0.2.0".to_string(),
      function: "get-stdin".to_string(),
  }
  ```
- Ningún código del componente llega a instanciarse ni a ejecutarse ante una denegación de capability.

---

## Contratos Públicos Propuestos en `crates/runtime`

```rust
impl HostPolicy {
    /// Concede acceso a la interfaz de salida estándar WASI 0.2.
    pub fn allow_wasi_stdout(&mut self) -> &mut Self;

    /// Concede acceso a la interfaz de salida de errores WASI 0.2.
    pub fn allow_wasi_stderr(&mut self) -> &mut Self;

    /// Concede acceso a la interfaz de entrada estándar WASI 0.2.
    pub fn allow_wasi_stdin(&mut self) -> &mut Self;

    /// Concede el trío completo de consola WASI (stdin, stdout, stderr) y streams.
    pub fn allow_wasi_stdio(&mut self) -> &mut Self;
}
```

---

## Acceptance Criteria

- **AC-01 (Deny-by-default en interfaces WASI):** Por defecto (`HostPolicy::deny_all()`), cualquier componente que importe interfaces WASI (`wasi:cli/stdout`, `wasi:cli/stdin`, etc.) es rechazado con `RuntimeError::UnauthorizedImport`.
- **AC-02 (Permisos granulares e independientes):** Conceder `allow_wasi_stdout()` permite a un componente escribir en `stdout`, pero si el mismo componente intenta leer de `stdin`, falla explícitamente sin ejecutar la lectura.
- **AC-03 (Expansión de macro-capability en manifiesto):** Un manifiesto declarando capability `stdio` autoriza las interfaces de consola requeridas, validado tanto por deserialización YAML como programática.
- **AC-04 (Protección contra capacidades no solicitadas):** Conceder capacidades de consola no habilita ni resuelve importaciones de filesystem (`wasi:filesystem`) ni red (`wasi:sockets`), que siguen siendo denegadas.
- **AC-05 (Suite de pruebas de seguridad y permisos):** Suite en `crates/runtime/tests/wasi_policy.rs` cubriendo:
  1. Componente con stdout autorizado -> ejecuta y captura salida con éxito.
  2. Componente con stdout no autorizado -> falla pre-ejecución con error tipado.
  3. Componente con stdout concedido intentando acceder a stdin -> falla con error tipado.
  4. Manifiesto YAML con categoría `stdio` aplicado a un `Tenant`.
