---
type: discovery
subtype: architecture-notes
generated_by: architecture-agent
---

# Architecture Notes (discovery)

Notas de descubrimiento complementarias a `knowledge/tech/current-state.md`. Son material de trabajo, no decisiones.

## Observaciones estructurales

- La POC usó 5 crates (`irtool`, `orchestrator`, `runner`, más dos CLIs). El acople observado es:
  - `orchestrator` concentra tanto el agente LLM como la publicación atómica — romperlo requiere definir el límite exacto entre "coordinar build" y "coordinar publicación".
  - `runner` mezcla la carga de WASM, la ejecución de suites YAML y el host polyfill. El runtime autónomo debería exportar las tres piezas como APIs separadas.
- `wit/kaddo:canal@0.1.0` es un fixture de prueba, no un contrato de dominio. Debe quedar explícitamente fuera de los contratos públicos del producto.

## Riesgos estructurales

- **Riesgo:** replicar la fusión `orchestrator = agente + publicador` en el crate `compiler` del MVP puede volver a mezclar responsabilidades. Mitigar manteniendo módulos internos (`compiler::implementation`, `compiler::publish`) con fronteras claras aunque compartan crate.
- **Riesgo:** el runner de la POC recompila dependencias en cada ejecución (open question L147). Reproducir ese comportamiento en el runtime nuevo invalida los objetivos de latencia; se necesita un resolver reutilizable antes de cerrar MVP.
- **Riesgo:** el fixture `kaddo:canal` puede filtrarse como "dependencia implícita" si no se separa en una carpeta aislada bajo `tests/` o `examples/`.

## Flujos a validar

1. **Build end-to-end:** narrativa Markdown → `System Spec` → IR → verificador → WASM Component → publicación con hash → ejecución reproducible por el runtime.
2. **Evolución v1 → v2:** reconciliar una regla cambiada, generar un artefacto nuevo, correr aceptación y confirmar que la versión previa queda accesible.
3. **Fallo seguro:** host capability no autorizada o import faltante → error explícito, nunca resultado `0`/`false`.

## Preguntas de arquitectura abiertas (resumen)

- Límite público vs privado del tipo `SystemSpec`.
- Formato del manifiesto de artefacto publicado y su esquema versionado.
- Mecanismo de firma/procedencia aceptable para el MVP de distribución.
- Estrategia de observabilidad mínima (logs estructurados vs tracing).
