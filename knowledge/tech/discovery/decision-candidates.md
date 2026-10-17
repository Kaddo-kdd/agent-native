---
type: discovery
subtype: decision-candidates
generated_by: architecture-agent
refined_by: roadmap-agent
---

# Decision Candidates (discovery)

Candidatos a ADR. **No son decisiones**: se materializan como ADR finales bajo `knowledge/tech/decisions/` mediante `decision-agent`. Esta lista está dividida en dos secciones:

- **Bloqueantes (RM-01 / INIT-01-03):** 5 ADRs que deben materializarse antes de abrir INIT-01-01 (workspace).
- **Diferidos:** candidatos que se materializan cuando su bloque los demande, no antes.

## Bloqueantes — a materializar en INIT-01-03

Prerrequisito del inicio efectivo de RM-02.

### [candidate] ADR-001 — Arquitectura inicial y dependencias entre módulos

- Contexto: workspace arranca con `crates/{ir,compiler,runtime}` + bin `cli`. La extracción de `orchestrator`/`specification` queda sujeta a evaluación en RM-04. El `compiler` **no** gestiona narrativa, LLM ni publicación.
- Decisión a tomar: límites de responsabilidad entre crates + criterio objetivo para extraer un crate de orquestación.
- Debe permitir explícitamente: cargar componentes WIT-compatibles independientes de su origen (objetivo arquitectónico diferido).
- Alternativas: 5 crates desde el día uno; monolito con módulos.
- Confianza: Medium.
- Sustento: A1 revisado, principio de separación de responsabilidades del roadmap.

### [candidate] ADR-002 — Runtime independiente del LLM y del compiler

- Contexto: el runtime no debe depender del compiler, del LLM ni de Kaddo. Lint CI debe hacer cumplir la frontera.
- Alternativas: runtime acoplado al compiler para simplificar metadatos (rechazada — rompe distribución autónoma).
- Confianza: High.
- Sustento: principio no negociable del roadmap.

### [candidate] ADR-003 — Modelo de Semantic IR v0.5 y política de compatibilidad

- Contexto: fijar el modelo v0.5 estable y el camino de evolución hacia v0.6 (`string`, errores) bajo flag experimental. **Selección exacta de versión de Wasmtime permanece abierta** hasta validar compatibilidad y restricciones.
- Alternativas: cerrar baseline Wasmtime ahora (rechazada por falta de evidencia); extender v0.5 directamente (rechazada por "equivalencia funcional primero").
- Confianza: Medium.
- Sustento: OQ L143, L144, L145.

### [candidate] ADR-004 — Estrategia de seguridad mínima transversal

- Contexto: fail-explicit en imports/capabilities faltantes, deny-by-default, verificación SHA-256 presentes desde RM-02/RM-03. RM-06 refina (fuel, policy loader completo, firma), no inaugura.
- Alternativas: diferir seguridad a RM-06 (rechazada por el ajuste "seguridad desde el inicio").
- Confianza: High.
- Sustento: principio del roadmap; Risk R-10, R-11.

### [candidate] ADR-005 — Modelo de memoria y Canonical ABI

- Contexto: el supuesto previo sobre dlmalloc fue retirado. Documentar asignación, liberación, crecimiento de memoria, Canonical ABI y ciclo de vida de instancias con base técnica y experimentos (INIT-02-05).
- Decisión a tomar: estrategia inicial de memoria con criterios para revisarla una vez haya mediciones (INIT-03-06).
- Alternativas: adoptar dlmalloc por defecto sin benchmark (rechazada); forzar instancias nuevas siempre (opción conservadora válida pre-INIT-06-03).
- Confianza: Low — requiere experimentos.
- Sustento: Risk R-11, OQ L146, OQ L148 reabierta.

## Diferidos — a materializar cuando su bloque los demande

### [candidate] Publicación atómica + evolución versionada (RM-04)

- Contexto: hash SHA-256, puntero activo, rollback atómico. Implementación inicial en la capa de orquestación, **sin** crate adicional salvo justificación.
- Momento de materialización: INIT-04-05.
- Confianza: High en el patrón; Medium en la ubicación modular.

### [candidate] CLI sin lógica de dominio (RM-05)

- Contexto: subcomandos invocan APIs públicas de los crates; suites automatizadas no parsean texto humano.
- Momento de materialización: INIT-05-01.
- Confianza: High.

### [candidate] Empaquetado, firma, publicación y verificación (RM-05)

- Contexto: A3 es objetivo, no decisión técnica cerrada. Firma (minisign vs cosign) y empaquetado se formalizan en ADR dedicada dentro de RM-05.
- Momento de materialización: INIT-05-03.
- Confianza: Low — depende de spike previo.

### [candidate] Resolver reutilizable de dependencias (RM-03)

- Contexto: reemplazar la recompilación por invocación observada en la POC con un resolver cacheado y reset explícito por tenant. Requiere línea base previa (INIT-03-06).
- Momento de materialización: INIT-03-02.
- Confianza: Medium.

### [candidate] `_reference/` como fuente histórica congelada (RM-01)

- Contexto: ya cubierto operativamente por INIT-01-04; la ADR formaliza la política para humanos y agentes.
- Momento de materialización: puede incluirse en INIT-01-04 como entregable documental, sin requerir un ADR separado.
- Confianza: High.

### [candidate] Interoperabilidad con componentes WIT-compatibles externos (objetivo diferido)

- Contexto: el runtime debe cargar componentes WIT-compatibles independientes de su origen. No se diseña un runtime cerrado. No requiere iniciativa activa en el roadmap actual.
- Momento de materialización: cuando exista demanda; INIT-03-01 ya incluye como criterio no-funcional cargar un componente WIT externo verificable.
- Confianza: Medium — depende de validación por INIT-03-01.

## Trazabilidad

| Decisión | Impacta a |
|---|---|
| ADR-001 | INIT-01-01, INIT-02-02, INIT-04-05, INIT-05-02, interop diferida |
| ADR-002 | INIT-03-01, INIT-04-05, INIT-05-02 |
| ADR-003 | INIT-02-01, INIT-02-04, INIT-07-02 |
| ADR-004 | INIT-02-04, INIT-03-04, INIT-03-05, RM-06 |
| ADR-005 | INIT-02-05, INIT-06-02, INIT-06-03 |
