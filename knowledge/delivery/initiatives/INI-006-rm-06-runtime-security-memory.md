---
type: initiative
id: INI-006
title: RM-06 — Runtime Security & Memory
status: planned
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
    status: in-progress
    materialized_as: AN-035 (in-progress), AN-036 (completed), AN-037 (completed), AN-038 (draft)
  - id: INIT-06-03
    title: Ciclo de vida de instancias y aislamiento multi-tenant
    status: candidate
  - id: INIT-06-04
    title: Verificación de integridad y procedencia en carga
    status: candidate
  - id: INIT-06-05
    title: Fixtures de ejecución extrema
    status: candidate
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
   - *Estado actual:* Límites de memoria y tablas verificados en los escenarios cubiertos en todas las plataformas (AN-036). Agotamiento de fuel verificado en Linux y macOS; en Windows MSVC permanece como problema observado y brecha abierta en investigación activa (AN-035 in-progress, AN-038 draft).
3. Test de contaminación cruzada entre tenants pasa con aislamiento, falla sin él.
4. `run --verify` falla con componente alterado; pasa con firma válida.
5. Fixtures de ejecución extrema disponibles y cubiertos en CI (Completado: AN-037).

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
5. **Brecha de contención de fuel en Windows MSVC:** En Wasmtime 27, el agotamiento de fuel en Windows desencadena `panic_cannot_unwind` (`0xc0000409`). No se declara completada la contención de CPU en Windows hasta que AN-038 valide una actualización de Wasmtime o un worker aislado por proceso.
6. **Invariante no negociable de aislamiento:** Prohibición estricta de reutilización de instancias entre tenants (Store fresco por invocación garantizado).
