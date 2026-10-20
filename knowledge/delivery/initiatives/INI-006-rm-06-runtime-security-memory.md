---
type: initiative
id: INI-006
title: RM-06 — Runtime Security & Memory
status: completed
completed_at: '2026-10-20'
knowledge_level: K4
domains:
  - Runtime & Memory
  - Security
related_capabilities:
  - Host Capability Policy
  - Independent Runtime Execution
roadmap_block: RM-06
roadmap_initiatives:
  - INIT-06-01
  - INIT-06-02
  - INIT-06-03
  - INIT-06-04
  - INIT-06-05
created_at: '2026-10-17'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-06-01
    title: Policy loader de capabilities (manifest-driven)
    status: completed
    materialized_as: AN-033, AN-034
  - id: INIT-06-02
    title: Límites CPU/memoria (fuel + instance limits)
    status: completed
    materialized_as: AN-035 (completed), AN-036 (completed), AN-037 (completed), AN-038 (completed)
  - id: INIT-06-03
    title: Ciclo de vida de instancias y aislamiento multi-tenant
    status: completed
    materialized_as: AN-039 (completed), AN-040 (completed)
  - id: INIT-06-04
    title: Verificación de integridad y procedencia en carga
    status: completed
    materialized_as: AN-041 (completed), AN-042 (completed)
  - id: INIT-06-05
    title: Fixtures de ejecución extrema
    status: completed
    materialized_as: AN-043 (completed)
---

# RM-06 — Runtime Security & Memory

## Goal

Profundizar seguridad y memoria **sobre** los controles mínimos ya introducidos en RM-02/RM-03. Convertir el runtime en apto para ejecutar código generado por IA con garantías verificables de aislamiento, límites y procedencia.

## Expected Value

Prerrequisito para habilitar I/O real (RM-07) y para presentar el runtime en escenarios multi-tenant o con componentes externos no confiables.

## Scope

- Policy loader con allow-list por componente, deny-by-default (evolución de INIT-03-05).
- Fuel accounting + `InstanceLimits` configurables con trap verificable.
- API `Tenant` + aislamiento multi-tenant con garantías antes de habilitar reutilización.
- `agent-native run --verify` con SHA-256 y verificación de firma del release.
- Fixtures WASM de ejecución extrema (recursión controlada, allocación masiva, errores inducidos) escritos a bajo nivel.

## Out of Scope

- Procedencia tipo in-toto / SLSA (futuro).
- Modelo de permisos tipo capabilities POSIX avanzadas (futuro).
- Concurrencia desde dentro del guest (RM-08 lo cubre desde el host).

## Success Criteria

1. Policy loader rechaza capabilities no listadas con error tipado; auditado por suite de pruebas negativas (Completado: AN-033, AN-034).
2. Fuel overflow produce trap determinista tipado (`RuntimeError::FuelExhausted`); memory limit excedido produce trap tipado (`RuntimeError::MemoryLimitExceeded`).
   - *Estado actual:* Límites de memoria y tablas verificados cross-OS (AN-036). Agotamiento de fuel verificado y contenido de forma limpia en Linux, macOS y Windows MSVC sin terminar el proceso anfitrión tras la promoción técnica a Wasmtime 29.0.1 (Bytecode Alliance PR #9710) en AN-038. Suite de tests 100% pasando en CI con 0 exclusiones `#[ignore]` (Completado: AN-035, AN-036, AN-038). Se registra deuda de mantenimiento para evaluar Wasmtime 49+/50+ en RM-08.
3. Test de contaminación cruzada entre tenants pasa con aislamiento, falla sin él. (Completado: AN-039, AN-040; validado empíricamente en `multi_tenant_isolation.rs` para los 5 escenarios E1-E5 de blast radius bajo Store efímero de ADR-005).
4. `run --verify` falla con componente alterado; pasa con firma válida. (Completado: AN-041, AN-042; validado determinísticamente con SHA-256 y Minisign Ed25519 offline en crates/cli/tests/run_verify.rs).
5. Fixtures de ejecución extrema disponibles y cubiertos en CI (Completado: AN-037, AN-043; recursión controlada, estrés de memoria y trampas inducidas verificadas cross-OS).

## Dependencies

- INIT-03-05 (controles mínimos) completado.
- INIT-02-05 (ADR-005 modelo de memoria) cerrado.
- INIT-04-05 e INIT-05-03 (manifiestos y pipeline de release) para INIT-06-04.

## Work Item Candidates

Ver `candidates:`.

## Open Questions

- OQ L149 (modelo de permisos, firma, procedencia) — se cierra en INIT-06-01/06-04.
- Riesgo R-11 (allocator incremental de la POC) — mitigado por INIT-02-05 + INIT-06-02/06-03.

## Learning y Precisiones Técnicas (INIT-06-02 / 2026-10-19)

1. **Fuel no equivale a tiempo de CPU de reloj:** Wasmtime limita trabajo computacional determinista mediante conteo de instrucciones abstractas. Los tiempos de reloj deben registrarse como mediciones empíricas observadas, nunca como garantías rígidas universales.
2. **Alcance real de `max_memory_bytes`:** Se evalúa en cada solicitud individual de crecimiento de memoria lineal WebAssembly (`ResourceLimiter::memory_growing()`) en el `Store` activo. No constituye una cuota acumulada de todas las memorias del Store, ni una cuota agregada o acumulativa entre múltiples invocaciones de un tenant, ni garantiza el aislamiento de toda la memoria del proceso anfitrión.
3. **Mapeo de errores y diagnóstico heurístico:** `ResourceLimiter` instruye a `memory.grow` a devolver `-1`. Cuando el guest entra en trap consecuente (`unreachable`), el runtime mapea heurísticamente el error a `RuntimeError::MemoryLimitExceeded` basándose en la bandera de denegación del Store. Un trap posterior no demuestra formalmente que la causa raíz haya sido el fallo de memoria. En INIT-06-03 se refinará el diagnóstico para desacoplar trampas no relacionadas de fallos de memoria previos.
4. **Variantes con host context:** Actualmente `invoke_*_with_host_ctx()` opera bajo rutas unmetered. INIT-06-03 unificará las rutas de ejecución bajo límites y cuotas homogéneas.
5. **Resolución de la contención de fuel en Windows MSVC:** Tras reproducir el aborto no controlado (`0xc0000409`) en Wasmtime 27 y 28 debido al uso de `longjmp` en libcalls, la actualización técnica a Wasmtime 29.0.1 (Bytecode Alliance PR #9710) migró los libcalls al retorno estructurado de trap status, permitiendo a Cranelift y Wasmtime capturar `Trap::OutOfFuel` en Windows de forma idéntica a Linux/macOS sin abortar el host. Todas las pruebas corren habilitadas en CI. Se mantiene registrada la deuda técnica para actualizar a Wasmtime 49+/50+ en RM-08.
6. **Invariante no negociable de aislamiento:** Prohibición estricta de reutilización de instancias entre tenants (Store fresco por invocación garantizado).
