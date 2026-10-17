---
id: AN-001
type: chore
status: completed
completed_at: 2026-10-17
title: Gobierno de _reference/ y política de solo lectura de la POC
knowledge_level: K2
suggested_knowledge_level: K2
scope_confidence: high
code:
  - .gitignore
  - knowledge/tech/reference-policy.md
  - knowledge/tech/codebase.md
  - knowledge/tech/current-state.md
source_roadmap_initiative: INIT-01-04
source_work_item_candidate:
  - WI-CANDIDATE-01-04-01
  - WI-CANDIDATE-01-04-02
related_domain: Foundation & Governance
domains:
  - Foundation & Governance
related_capabilities: []
expected_value: >-
  Garantizar que la POC `_reference/poc-agent-native/` quede como fuente
  histórica consultable sin convertirse en dependencia del build ni del producto
  Agent-Native.
risks:
  - >-
    R-05-governance: indexadores (IDE, agentes) tratan la POC como código del
    producto si solo se depende de .gitignore
dependencies: []
source_signals:
  - >-
    Risk: la POC indexada como código del producto introduce ambigüedad y riesgo
    de deriva
  - 'Assumption: _reference/poc-agent-native/ es solo lectura (codebase.md)'
decision_candidates: []
related_decisions: []
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
ready_at: '2026-10-17'
---

# AN-001 — Gobierno de `_reference/` y política de solo lectura de la POC

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

**Actor and outcome:**
Mantenedores y agentes del repositorio `Kaddo-kdd/agent-native` disponen de la POC `_reference/poc-agent-native/` como fuente histórica consultable, sin que sus archivos entren al índice de Git ni a los artefactos del producto. La política está documentada y verificable.

**Current behavior:**
El repositorio nuevo contiene únicamente `LICENSE`. No existe `.gitignore` ni política documentada sobre `_reference/`. Si se clona la POC en esa ruta, Git la indexaría, `cargo` podría intentar incluirla al existir un workspace, y los agentes podrían leerla y modificarla sin restricción explícita.

**Target behavior:**
- `_reference/` está excluida del índice de Git y de cualquier `cargo metadata` futuro.
- Existe `knowledge/tech/reference-policy.md` que:
  1. Establece que `_reference/poc-agent-native/` es de solo lectura y no puede modificarse como parte de los Work Items del producto nuevo.
  2. Prohíbe dependencias de compilación o ejecución hacia `_reference/` (ni rutas, ni crates como path dependencies, ni artefactos internos de la POC).
  3. Autoriza a los agentes consultar selectivamente la POC para recuperar comportamiento, contratos y evidencia, con la obligación de implementar exclusivamente dentro de los módulos del nuevo producto.
  4. Declara explícitamente que `.gitignore` no impide que un agente o IDE lea o modifique archivos; se requieren instrucciones explícitas de trabajo que esta política suministra.
  5. Documenta los procedimientos de verificación aplicables (`git check-ignore`, `git status`, `cargo metadata` cuando exista workspace Rust) con sus resultados esperados.
- La política está firmada por humano y referenciada desde `knowledge/tech/codebase.md` y `knowledge/tech/current-state.md`.

**Entry points:**
- `.gitignore` en la raíz del repositorio.
- `knowledge/tech/reference-policy.md` como documento canónico de la política.
- Referencias desde `knowledge/tech/codebase.md` y `knowledge/tech/current-state.md`.

**End-to-end flow:**
1. Un humano o agente clona `Kaddo-kdd/agent-native` localmente.
2. Clona `Trycatch-tv/poc-agent-native` dentro de `_reference/poc-agent-native/`, fijada en commit `26b6a18e2f4a50086d5ee604051154d9ea77d768`.
3. `git status --short` no lista archivos bajo `_reference/`.
4. `git check-ignore -v _reference/poc-agent-native/Cargo.toml` reporta la regla que la ignora.
5. Un agente consulta un archivo de `_reference/` para recuperar comportamiento, pero cualquier implementación se escribe exclusivamente bajo `crates/`, `wit/`, `tests/`, `examples/` o `docs/` del producto nuevo.
6. Si existe workspace Rust (post INIT-01-01), `cargo metadata --format-version=1` no lista paquetes bajo `_reference/`.

**Problem:**
Sin exclusión explícita y política documentada, la POC se mezcla con el producto nuevo y los agentes pueden tratarla como código editable, lo que rompe la invariante de "migración por extracción, no por import" y hereda deuda experimental de la POC.

**Expected result:**
El repositorio presenta la POC localmente para consulta, pero el producto nuevo es cerrado respecto a ella en Git, en cargo y en las instrucciones de agentes.

**Suggested Knowledge Level:** K2

**Impact analysis:**
- Product / UI: not-applicable
- Frontend: not-applicable
- Backend: not-applicable (no hay backend aún)
- Database: not-applicable
- Configuration: **affected** — `.gitignore` nuevo
- Feature flags: not-applicable
- Content / copy: **affected** — `knowledge/tech/reference-policy.md` nuevo; actualizaciones menores en `codebase.md` y `current-state.md`
- Authentication / authorization: not-applicable
- Notifications: not-applicable
- Analytics: not-applicable
- Documentation: **affected** — política de referencia
- Operations / release: **reviewed-not-affected** — no hay pipeline aún; INIT-05-03 debe tomar en cuenta esta política al configurarse

**Module coverage:**
Agent-Native es monorepo con los crates todavía por crear (INIT-01-01). Los módulos conocidos y planeados se evalúan así:
- `crates/ir` (planeado): not-applicable (aún no existe)
- `crates/compiler` (planeado): not-applicable
- `crates/runtime` (planeado): not-applicable
- `crates/cli` (planeado): not-applicable
- `knowledge/`: **affected** — nuevo archivo `tech/reference-policy.md`
- Repositorio raíz: **affected** — `.gitignore`

**Scope unknowns:**
- No se decide todavía la forma de obtener localmente la POC (submódulo vs clone manual). La política debe cubrir ambos escenarios o declarar uno canónico; propuesta inicial: clone manual, sin submódulo (evita `.git` embebido en el repo nuevo).
- No se define todavía si se añade un hook pre-commit que bloquee commits bajo `_reference/`; se registra como "fuera de alcance" (ver abajo).

**Scope confidence:** high

Razones:
- La iniciativa está acotada a tres artefactos (`.gitignore`, documento de política y referencias cruzadas).
- No introduce código ejecutable ni toca módulos del producto que aún no existen.
- Las verificaciones son objetivas y están cubiertas por comandos estándar.

## Acceptance criteria

1. **POC disponible localmente para consulta.** Con la POC clonada en `_reference/poc-agent-native/@26b6a18`, los archivos pueden leerse con herramientas del SO y agentes; no se requiere nada más que su presencia en disco.
2. **Exclusión en Git verificada.**
   - `git check-ignore -v _reference/poc-agent-native/Cargo.toml` imprime la línea `.gitignore` que la cubre.
   - `git status --short` tras clonar la POC no lista ningún archivo bajo `_reference/`.
   - `git ls-files _reference/` devuelve salida vacía.
3. **Política documentada.** `knowledge/tech/reference-policy.md` existe, está en español, y cubre al menos: solo lectura, prohibición de dependencias de compilación/ejecución, autorización de consulta por agentes, obligación de implementar en módulos del producto nuevo, aclaración de que `.gitignore` no impide lecturas/escrituras automáticas, y procedimientos de verificación con resultados esperados.
4. **Prohibición de dependencias técnicas.** El documento declara explícitamente que ningún `Cargo.toml` del workspace puede incluir una dependencia `path = "../_reference/..."` o similar; cualquier intento futuro debe fallar en revisión.
5. **Verificación post-workspace.** El documento establece que, una vez exista un workspace Rust (INIT-01-01), `cargo metadata --format-version=1` no debe listar paquetes bajo `_reference/`. Este comando queda documentado aunque todavía no se pueda ejecutar.
6. **Referencias cruzadas.** `knowledge/tech/codebase.md` (sección "Repository structure" > "Estructura objetivo provisional") y `knowledge/tech/current-state.md` (sección "Modules" > "_reference/poc-agent-native/") enlazan a `reference-policy.md`.

## Out of scope

- Hook `pre-commit` que bloquee modificaciones en `_reference/` (opcional, se evalúa en una iniciativa posterior).
- Automatización de la clonación de la POC (puede añadirse como script en un WI posterior).
- Decidir el baseline exacto de Wasmtime, el modelo de memoria o cualquier otra decisión arquitectónica — pertenecen a INIT-01-03 (ADRs bloqueantes).
- Crear el workspace Cargo — pertenece a INIT-01-01.

## Validation

Sin implementación todavía: la validación se ejecutará por el humano revisor y por el implementation-agent **cuando** este WI pase a `in-progress`. Pasos previstos:

1. Clonar la POC en el host local: `git clone https://github.com/Trycatch-tv/poc-agent-native _reference/poc-agent-native && (cd _reference/poc-agent-native && git checkout 26b6a18e2f4a50086d5ee604051154d9ea77d768)`.
2. Ejecutar y capturar salida esperada:
   - `git check-ignore -v _reference/poc-agent-native/Cargo.toml` → una línea con `.gitignore:<n>:/_reference/ _reference/poc-agent-native/Cargo.toml`.
   - `git status --short` → vacío (o solo archivos del cambio en revisión, nunca bajo `_reference/`).
   - `git ls-files _reference/` → vacío.
3. Lectura humana: abrir `knowledge/tech/reference-policy.md` y verificar que los 6 criterios de aceptación 3 están cubiertos.
4. Revisión manual: `knowledge/tech/codebase.md` y `knowledge/tech/current-state.md` referencian la política.
5. Verificación diferida (una vez exista workspace Rust): `cargo metadata --format-version=1 | jq '.packages[].manifest_path' | grep _reference` → sin coincidencias.
6. `kaddo guard` sobre el cambio no debe reportar deriva en artefactos no relacionados.
7. `kaddo verify AN-001` (comando previsto por Kaddo cuando el WI esté `completed`) ejerce los criterios de aceptación anteriores sobre la evidencia registrada.

## Definition of Done

- `.gitignore` incluye `/_reference/` (y comentario explicando el porqué).
- `knowledge/tech/reference-policy.md` creado y cumple los 6 criterios de aceptación.
- `knowledge/tech/codebase.md` y `knowledge/tech/current-state.md` enlazan a la política.
- Los comandos de verificación 2.1–2.3 producen la salida esperada en el host del revisor.
- El WI tiene evidencia asociada (output de los comandos, enlace al diff) y pasa a `completed` tras `kaddo verify`.

**Open questions:**

- OQ-AN-001-01: ¿se añade un hook `pre-commit` opcional ahora o se difiere a una iniciativa posterior? Propuesta: diferir (fuera de alcance).
- OQ-AN-001-02: ¿la clonación de la POC se automatiza mediante script bajo `scripts/`? Propuesta: diferir hasta tener workspace y CI.
- OQ-AN-001-03: ¿la política contempla también agentes externos (p. ej. GitHub Copilot, Cursor) de forma explícita, o se mantiene genérica? Propuesta: redacción genérica, con ejemplos nombrando IDE/agentes.

**Suggested ownership (code globs):**

- `.gitignore`
- `knowledge/tech/reference-policy.md`
- `knowledge/tech/codebase.md`
- `knowledge/tech/current-state.md`

**Related domain / capability:**
- `related_domain`: Foundation & Governance (nuevo dominio candidato — no existe aún en `knowledge/product/capabilities.md`; se registra como candidato de dominio en el refinamiento futuro).

**Related decisions:**
- `decision_candidates`: ninguno bloqueante. Este WI no depende de ADR-001..005.

**Related legacy findings:** no aplica — el proyecto no está en estado `legacy`; `_reference/` no es "legacy" sino "POC de referencia".

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-01 — Foundation & POC Assessment |
| Roadmap iniciativa | INIT-01-04 — Gobierno de `_reference/` y `.gitignore` |
| Candidatos consolidados | WI-CANDIDATE-01-04-01, WI-CANDIDATE-01-04-02 |
| Business goals | #6 (proyecto open source independiente) |
| Capacidad producto | Foundation & Governance (transversal) |
| Principio del roadmap | #1 (Recuperar antes de innovar), #6 (Separación de responsabilidades) |

## Learning

**Lo que salió como se esperaba:**

- Los tres criterios ejecutables (`git check-ignore`, `git ls-files _reference/`, `git status --short`) pasan a la primera. El patrón `/_reference/` cubre tanto la carpeta como todos sus descendientes.
- Consolidar los dos candidatos (`.gitignore` + política documentada) en un único WI fue correcto: ambos entregables comparten criterios de aceptación y validarlos por separado habría introducido fricción artificial.
- La política como documento canónico separado (`reference-policy.md`) referenciado desde `codebase.md` y `current-state.md` evita duplicación: cualquier futura actualización vive en un solo lugar.

**Lo inesperado:**

- `kaddo create --from roadmap`, `kaddo ready` y `kaddo learn` fallan por `ERR_TTY_INIT_FAILED` cuando se ejecutan desde un entorno sin TTY interactivo (como esta sesión de agente). Los efectos se reprodujeron manualmente respetando el schema: frontmatter con `status`/`completed_at`, carpeta de lifecycle correcta, campos de trazabilidad VS-077/VS-078, secciones H2 (`## Acceptance criteria`, `## Validation`, `## Definition of Done`, `## Learning`). Kaddo reconoce los WIs resultantes de forma idéntica (`kaddo history`, `kaddo verify`).
- La inicialización del YAML frontmatter con listas que contenían valores con `:` (p. ej. `- Risk: ...`) rompió el parser; hay que citar los strings. Vale como norma para futuros WIs.
- Kaddo expone `suggested_knowledge_level` en el agente, pero `kaddo ready` lee `knowledge_level`. Ambos campos conviven en el WI.
- La verificación automatizada de Kaddo marcó los 3 AC con comando como `not-verified` porque el CLI no ejecuta shell. La verificación manual pasó; no es un bloqueador funcional, pero conviene documentarlo cuando se diseñe el `implementation-agent` real.

**Riesgos materializados:** ninguno. R-05-governance (indexadores que lean/escriban `_reference/`) sigue vigente — la política regla 6/7 lo mitiga, pero `.gitignore` por sí solo no es suficiente, y esto quedó documentado y verificable.

**Deuda y seguimientos:**

- OQ-AN-001-01, OQ-AN-001-02 y OQ-AN-001-03 quedan como preguntas de la política, no del WI. Diferidas hasta que surja un WI o incidente que las toque.
- El dominio "Foundation & Governance" sigue como candidato en el roadmap y en el WI, pero no está formalmente en `capabilities.md`. Vive como pendiente al refinar el modelo de dominios cuando `capabilities.md` pase al formato `## Capability Domains`.
- El criterio de aceptación 5 (`cargo metadata`) permanece como "verificación diferida" hasta que exista workspace (INIT-01-01).

**Lo que esta iniciativa establece para las siguientes:**

- Toda implementación posterior puede asumir que `_reference/` está excluida del build y de Git, con política vinculante para humanos y agentes.
- La matriz de extracción (INIT-01-02) puede consultar la POC con reglas claras: lectura selectiva, implementación exclusivamente en el producto nuevo, cita obligatoria del archivo consultado en el WI.
- Los ADRs bloqueantes (INIT-01-03) deben respetar la regla 5 (interoperabilidad conservada): el runtime no se diseña cerrado a componentes externos WIT-compatibles.
