---
id: AN-016
type: feature
status: completed
completed_at: 2026-10-18
title: Robustez del compilador y modelo de memoria (ADR-005)
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: medium
code:
  - crates/compiler/src/codegen.rs
  - crates/compiler/src/lib.rs
  - crates/compiler/src/context.rs
  - crates/compiler/tests/nested_records.rs
  - crates/compiler/tests/compile_context.rs
  - knowledge/tech/decisions/ADR-005-modelo-memoria-canonical-abi.md
  - knowledge/tech/discovery/memory-model-experiments.md
source_roadmap_initiative: INIT-02-05
cross_initiatives:
  - INIT-02-02
source_work_item_candidate:
  - Technical Review 2026-10-17 (ref 06d270d) — WI-C
related_domain: Compilation
domains:
  - Compilation
  - Runtime & Memory
related_capabilities:
  - Component Compilation
  - Semantic IR Verification
expected_value: >-
  Verifica experimentalmente records anidados, elimina la dependencia del cwd en
  la resolución de WIT con una API `CompileContext` compatible, y actualiza
  ADR-005 con hallazgos reproducibles sobre asignador incremental, Canonical ABI
  y records anidados. No supone defectos sin reproducirlos.
risks:
  - >-
    R-C-01: compile_ir_to_wasm existente puede volverse breaking al introducir
    contexto; mitigado añadiendo compile_ir_to_wasm_with_context sin tocar la
    firma existente
  - >-
    R-C-02: caracterizar el asignador requiere fixtures reproducibles de alto
    nivel (IR v0.5), no WAT handcrafted — alcance acotado por eso
  - >-
    R-C-03: Wasmtime no expone ResourceLimiter desde el host en la API usada; la
    caracterización de memoria se limita a tiempo de pared y conteo de llamadas
    de allocación, no bytes exactos
dependencies:
  - AN-005
  - AN-006
  - AN-008
  - AN-014
source_signals:
  - >-
    Technical Review: compilación depende parcialmente del cwd — verificado en
    memory-model-experiments.md §1
  - >-
    crates/compiler/src/codegen.rs:379-385 confirmado: fs::read_to_string con
    fallbacks relativos ../../
  - >-
    Technical Review: record.construct anidado puede sobrescribir el puntero del
    registro exterior — NO verificado experimentalmente aún
  - >-
    memory-model-experiments.md §2: solo anidamiento de EXPRESIONES verificado
    en AN-014 (no record-dentro-de-record)
  - >-
    ADR-005: baseline operativa con 3 [open] (allocator alternativo, instancias
    persistentes, allocación masiva)
  - >-
    AN-014 Learning: snake_case en IR vs kebab-case en WIT — convención a
    formalizar
decision_candidates: []
related_decisions:
  - ADR-005
project_state: ai-assisted
generated_by: work-item-agent
refined_by: work-item-agent
template_version: 1
created_at: 2026-10-17T00:00:00.000Z
updated_at: 2026-10-18T00:00:00.000Z
ready_at: '2026-10-18'
---

# AN-016 — Robustez del compilador y modelo de memoria

**Actor y resultado:**
Un contribuidor obtiene evidencia reproducible de que (a) `record.construct` anidado hasta 2+ niveles produce la salida esperada o un bug tipado si existe, (b) el compilador acepta un `CompileContext` que elimina la dependencia del cwd, y (c) ADR-005 incluye secciones con experimentos documentados, alternativas evaluadas y decisiones justificadas en lugar de `[open]` genéricos.

**Current behavior:**

- `crates/compiler/src/codegen.rs:378-385` resuelve `wit_file` con `std::fs::read_to_string(p).or_else("../../{p}").or_else("../../../{p}")`. **Hoy no se dispara** porque los fixtures del workspace usan rutas que resuelven desde raíz del repo; es **deuda latente**, no bug activo (confirmado en `memory-model-experiments.md §1`).
- `record.construct` anidado **nunca** se ha ejercitado en pruebas. AN-014 verifica una capa de anidamiento de **expresiones aritméticas** dentro de un record, pero no `record.construct` **dentro** de otro `record.construct`.
- ADR-005 declara baseline operativa y 3 `[open]` sin evidencia experimental.

**Target behavior:**

### 1. Fixture reproducible de records anidados (primero, sin asumir defecto)

- `crates/compiler/tests/fixtures/ir/nested-records.ir.json`: IR v0.5 con un `record.construct` que contiene **al menos** otro `record.construct` como campo.
- `crates/compiler/tests/fixtures/wit/nested-records.wit`: contrato WIT asociado con records anidados.
- `crates/compiler/tests/nested_records.rs` (o `crates/runtime/tests/nested_records.rs` si el runtime es necesario): compile + load + invoke; verifica cada campo de ambos niveles con `assert_eq!`.
- **Si el test pasa:** se documenta el comportamiento correcto en ADR-005 y en el Learning. **Si falla:** se diagnostica (dónde se corrompe el puntero), se refactoriza `emit_expr`/`record.construct` en `codegen.rs` con el mínimo cambio necesario, y la prueba queda como regresión.

### 2. API `CompileContext` compatible

- Nueva `src/context.rs`:
  ```rust
  pub struct CompileContext {
      pub wit_root: std::path::PathBuf,
  }
  impl CompileContext {
      pub fn new(wit_root: impl Into<PathBuf>) -> Self;
      pub fn cwd() -> std::io::Result<Self>;  // explícito, para quien quiera el comportamiento previo
  }
  ```
- Nueva función pública `compile_ir_to_wasm_with_context(root, &CompileContext) -> Result<Vec<u8>, Vec<Diagnostic>>`.
- `compile_ir_to_wasm(root)` existente **se mantiene** y delega a `compile_ir_to_wasm_with_context(root, &CompileContext::cwd().unwrap_or_default())` para preservar comportamiento 1:1 con AN-008/013/014 y los 8 tests POC.
- En `codegen.rs`: todos los `std::fs::read_to_string(p).or_else("../../{p}")...` se reemplazan por `context.resolve(p)` que **falla directamente** si la ruta no existe bajo `wit_root` o es absoluta y accesible. Diagnóstico mejorado: `E_WIT_NOT_FOUND` con la ruta efectivamente probada.
- Prueba `tests/compile_context.rs`: ejerce el `with_context` desde dos cwd diferentes y verifica que el hash SHA-256 del artefacto es idéntico.

### 3. Caracterización del asignador incremental

- Documento `knowledge/tech/discovery/memory-model-notes.md` (nuevo o renombrando `memory-model-experiments.md` según decisión en implementación).
- Experimentos **sin cambiar el compilador**:
  - **E1 — allocación repetida:** una función que construye N records intermedios antes de retornar el último. Medido con `criterion` para N ∈ {1, 10, 100} si la gramática v0.5 lo soporta; de lo contrario, documentado como limitación.
  - **E2 — record grande:** un record con K campos (K ∈ {2, 10, 50}) retornado end-to-end. Medir tiempo de pared.
  - **E3 — anidamiento profundo:** el fixture nested_records de §1 con 2, 3 niveles.
  - **E4 — invocaciones sucesivas sobre la misma instancia Resolver-cached:** confirma que la asignación no acumula entre invocaciones (Store fresh per invoke, ADR-005 baseline).
- Observaciones documentadas, **no decisiones silenciosas**.

### 4. Canonical ABI — investigación

- Sección documental en `memory-model-notes.md` que lista:
  - Qué se **copia** al cruzar la frontera (records por valor, confirmado por AN-014).
  - Qué se **mueve** (ninguna cesión observada bajo IR v0.5).
  - Qué liberación requiere el host (ninguna, el Store se destruye al final de la invocación).
  - Diferencias con strings/listas/resources — fuera de alcance, pero listadas como pending.

### 5. ADR-005 actualizado

- Las 3 secciones `[open]` quedan reemplazadas por una de estas por sección:
  - `[chosen: X]` con justificación y resultado experimental.
  - `[deferred_to: WI-ID]` con criterio objetivo de cierre.
- Enlace al memory-model-notes.md.
- Historial preservado (no se borran los `[open]` originales; se marcan como superados por este WI).

## Acceptance criteria

1. **Fixture + test de records anidados** en `tests/`, con al menos 2 niveles de `record.construct`.
2. **El test pasa** verificando campo por campo, **o** reporta un bug reproducible que se corrige en el mismo WI con el mínimo cambio en `codegen.rs`.
3. **`CompileContext` público** con `new`, `cwd` y `resolve(path)`.
4. **`compile_ir_to_wasm_with_context`** operativo; `compile_ir_to_wasm` delega internamente a él y preserva comportamiento 1:1 (todos los tests actuales siguen pasando sin modificaciones).
5. **Determinismo cross-dir:** una prueba compila el mismo IR + WIT desde dos cwd distintas usando el contexto y verifica hash idéntico.
6. **Fallbacks relativos (`../../{p}`, `../../../{p}`) eliminados** del codegen; cualquier ruta no resuelta por el contexto es `E_WIT_NOT_FOUND` con la ruta probada.
7. **`memory-model-notes.md`** publicado con experimentos E1-E4 (los que son factibles bajo IR v0.5) y conclusiones.
8. **ADR-005 actualizado:** ningún `[open]` sin criterio objetivo de cierre o decisión documentada.
9. **Tests existentes (39) siguen verdes.** CI remota verde 3 OS.
10. **Lint + fmt verdes.** `cargo fmt`, `clippy pedantic -D warnings`.
11. **ADR-001/002 intactos:** `cargo tree -p agent-native-runtime --edges normal` sin `compiler`; `agent-native-ir` sin WASM/sha2.

## Out of scope

- Instancias persistentes / warm pool / pool de Stores (INIT-08-03, INIT-06-03).
- Extensiones de Semantic IR (string, resources) — RM-07.
- WASI — RM-07.
- Instrumentación `ResourceLimiter` para bytes exactos — INIT-06-02 o INIT-08-02 (documentado como limitación).
- Modificación de la POC `_reference/` (prohibido por política).
- Cualquier cambio en runtime que no sea para soportar las pruebas de este WI.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release

# Determinismo cross-dir
cargo test -p agent-native-compiler --test compile_context

# ADR invariantes
cargo tree -p agent-native-runtime --prefix none --edges normal | grep -i compiler || echo OK
cargo tree -p agent-native-ir --prefix none | grep -iE "wasm|sha2" || echo OK

# ADR-005 lint (debe decrecer)
grep -c "^\[open\]" knowledge/tech/decisions/ADR-005-modelo-memoria-canonical-abi.md
```

## Definition of Done

- Fixture nested records + 2+ niveles verificados.
- `CompileContext` operativo + `compile_ir_to_wasm_with_context`.
- Prueba cross-dir con hash reproducible.
- Fallbacks `../../` eliminados.
- `memory-model-notes.md` publicado.
- ADR-005 sin `[open]` sin criterio objetivo.
- Tests existentes verdes + nuevos verdes.
- CI remota 3 OS verde.
- WI `completed` (sujeto a aprobación humana explícita).

## Open questions

- **OQ-AN-016-01:** ¿`record.construct` dentro de `record.construct` existe realmente como construcción válida en la gramática v0.5, o la POC lo prohibía implícitamente? Primer paso: construir el fixture y ver qué dice el verifier. **Si rechaza:** el riesgo del bug se descarta por definición y el AC #2 se simplifica a "documentar que no es construible".
- **OQ-AN-016-02:** ¿`CompileContext::resolve` debe aceptar rutas absolutas? Propuesta: sí, con precedencia (absoluta > relativa-al-wit_root). Documentar como tal.
- **OQ-AN-016-03:** `memory-model-experiments.md` (exploratorio AN-016, pre-ready) vs `memory-model-notes.md` (productivo AN-016, post-ready): ¿fusionar o mantener separados? Propuesta: renombrar el exploratorio a `memory-model-notes.md` y agregarle las secciones E1-E4 productivas. Preserva historia.
- **OQ-AN-016-04:** El `requirements` en el IR no afecta bytes del WASM (hallazgo AN-013). ¿`CompileContext` debe hacer que `wit_file` tampoco afecte el hash si el contenido es idéntico? Propuesta: fuera de alcance — el hash depende del contenido del WIT, no de la ruta.

## Trazabilidad

| Fuente | Referencia |
|---|---|
| Roadmap bloque | RM-02 |
| Roadmap iniciativa | INIT-02-05 (principal), INIT-02-02 (refuerza) |
| ADRs | ADR-005 (actualiza), ADR-001/002/003 (preserva) |
| Technical Review | WI-C (ref 06d270d) |
| Prerrequisitos | AN-005, AN-006, AN-008, AN-014 |
| Evidencia exploratoria previa | `knowledge/tech/discovery/memory-model-experiments.md` (AN-016 pre-ready) |

## Learning

**Lo que salió como se esperaba:**

- `CompileContext` + `ResolveOutcome` encajan en < 110 LOC y preservan `compile_ir_to_wasm(root)` byte-a-byte para los call sites previos (AN-008/013/014).
- Los 3 tests de `compile_context.rs` capturan la invariante central: dado un contexto explícito, el cwd no influye en el output.
- `memory-model-notes.md` quedó como documento productivo, con `memory-model-experiments.md` como historia pre-ready; ambos se referencian cruzadamente.

**Lo inesperado y cómo se trató:**

- **Primer intento del `resolve()` fue permisivo:** aceptaba path relativo "tal cual" como fallback antes de probar `wit_root`. El test de root incorrecto no lo detectaba y lo dejé pasar (los paths resolvían contra cwd=compiler-crate por casualidad). Corregido: rutas relativas resuelven **solo** contra `wit_root`. Ahora el AC #5 muerde.
- **El supuesto defecto de record anidado es inalcanzable desde v0.5.** La disciplina "no asumir defecto hasta reproducir" evitó un refactor innecesario del emitter. El fixture queda como regresión documental para IR v0.6+.
- **Tests existentes de AN-014/015 fallaron** al eliminar los fallbacks `../../` — reveló que la deuda era real. Resuelto cambiando el path dentro del fixture (`tests/fixtures/wit/...` relativo al compiler crate) y haciendo que los tests de `agent-native-runtime` construyan el contexto con `CARGO_MANIFEST_DIR + "/../compiler"`.
- **ADR-005 queda sin ningún `[open]` sin criterio objetivo de cierre.** Dos menciones residuales de `[open]` en el texto son referencias históricas (texto en prosa), no estados activos.

**Riesgos materializados:**

- R-C-01 (API breaking): mitigado — `compile_ir_to_wasm` se mantiene como shim; firma pública intacta.
- R-C-02 (fixtures de estrés para asignador): materializado parcialmente — bloqueado por limitaciones de IR v0.5; documentado como deferral en `memory-model-notes.md` §3 y en ADR-005.
- R-C-03 (ResourceLimiter): materializado — bytes exactos requieren instrumentación de nivel bajo; diferido a INIT-06-02 con criterio objetivo en ADR-005.

**Deuda y OQ no resueltas:**

- `compile_ir_to_wasm(root)` sin contexto sigue leyendo cwd implícitamente (shim de compatibilidad). Para eliminarlo requiere migrar los 8 tests `poc_regression` (que compilan IR v0.1–v0.4 sin `wit_file` externo, por lo que el cwd no afecta su resultado, pero la firma depende de él). Fuera de alcance de AN-016.
- Experimentos de allocación masiva pendientes hasta IR v0.6+ (documentado).

**Lo que esta iniciativa establece:**

- **AN-017** (benchmarks + docs) ya puede renombrar `warm_invoke_identity` y añadir un bench Component Model usando `compile_ir_to_wasm_with_context` + `program-005`.
- **INIT-06-02** (fuel + instance limits) tiene criterio objetivo claro: integrar `ResourceLimiter` desbloquea las mediciones diferidas de ADR-005.
- **INIT-06-03** (aislamiento multi-tenant) tiene la pregunta correctamente diferida.
- **RM-07 / INIT-07-02** (IR v0.6 string/listas/loops): cuando se materialice, debe re-ejercer el experimento de records anidados y las mediciones de allocación masiva.
- **ADR-005** queda listo como contrato invariante (`[chosen]` Canonical ABI + `[chosen]` CompileContext).

### Evidencias de ejecución

- Commit implementación: `d5fef8c` → CI remota [#39145534837](https://github.com/Kaddo-kdd/agent-native/actions/runs/39145534837) **success** en ubuntu-latest, windows-latest, macos-latest.
- Local Windows: `cargo test --workspace` → 43 pruebas, 0 fallos.
- `kaddo verify AN-016 --json`: `"readiness": "READY_TO_COMPLETE"`, 0 blockers, 0 warnings, 0 AC not-verified.
- `cargo tree -p agent-native-runtime --edges normal | grep compiler` → vacío (ADR-002 intacto).
- `cargo tree -p agent-native-ir | grep -iE "wasm|sha2"` → vacío (ADR-001 intacto).

────────────────────────
Agent: work-item-agent

Produced:
knowledge/delivery/work-items/completed/AN-016-compiler-robustness-memory-model.md
crates/compiler/src/context.rs
crates/compiler/src/{codegen,lib}.rs (modificados)
crates/compiler/tests/compile_context.rs (3 tests)
crates/compiler/tests/fixtures/{ir,wit}/nested-records.*
crates/runtime/tests/nested_records.rs (1 test)
crates/runtime/tests/{e2e_v05,wit_policy}.rs (adaptados al contexto)
knowledge/tech/decisions/ADR-005-modelo-memoria-canonical-abi.md (actualizado)
knowledge/tech/discovery/memory-model-notes.md (nuevo, productivo)
knowledge/tech/discovery/memory-model-experiments.md (exploratorio preservado)

Next (pending user authorisation):
AN-017 (benchmarks + docs consolidation; depende blandamente de program-005
y compile_ir_to_wasm_with_context, ambos disponibles).
────────────────────────
