---
type: discovery
subtype: memory-model-notes
work_item: AN-016
generated_by: implementation-agent
status: productive — experimentos ejecutados, informa ADR-005
date: 2026-10-18
supersedes: knowledge/tech/discovery/memory-model-experiments.md
---

# AN-016 — Memory model notes (post-implementación)

Resultados verificables de los experimentos autorizados en AN-016. Supersede el documento exploratorio `memory-model-experiments.md` (que registraba hipótesis pre-implementación) y lo preserva como historia.

## 1. Dependencia del cwd (resuelta)

**Hipótesis Technical Review:** `codegen.rs:378-385` depende del cwd mediante `std::fs::read_to_string(p).or_else("../../{p}").or_else("../../../{p}")`.

**Reproducción:** confirmada. Antes de AN-016, ejecutar `cargo test -p agent-native-runtime` producía uno de dos resultados según cwd:
- Desde raíz del workspace: fixtures resolvían (los paths son `crates/compiler/tests/fixtures/wit/...` desde raíz).
- Desde `crates/runtime/`: resolvían por casualidad vía `.or_else("../../{p}")`.

**Implementación:** `crates/compiler/src/context.rs` nuevo con `CompileContext { wit_root }`. `compile_ir_to_wasm_with_context(root, &ctx)` resuelve rutas **solo** contra `wit_root`, nunca contra cwd. `compile_ir_to_wasm(root)` existente delega a `CompileContext::default() = cwd`, preservando el comportamiento anterior **solo cuando la cwd es la correcta**.

**Verificación (`crates/compiler/tests/compile_context.rs`, 3 tests):**
- Mismo IR + mismo contexto → bytes y hash idénticos.
- Contexto apuntando a un root inexistente → `E_WIT_NOT_FOUND` con las rutas probadas enumeradas; **nunca** falls through.
- API explícita inmune al cwd por construcción.

**Fixtures actualizadas:** `wit_file` ahora es relativo al crate `agent-native-compiler` (`tests/fixtures/wit/<name>.wit`), no al workspace root. Los tests que las usan construyen el contexto con `env!("CARGO_MANIFEST_DIR")` + navegación al compiler crate.

## 2. `record.construct` anidado (defecto descartado por diseño)

**Hipótesis Technical Review:** un `record.construct` dentro de otro `record.construct` puede sobrescribir el puntero del registro exterior durante la evaluación.

**Reproducción:** se construyó un fixture `nested-records.{ir.json, wit}` con `record outer { inside: inner, tag: i32 }`. **El verifier de Semantic IR v0.5 lo rechaza antes de llegar a codegen:**

```
E_UNKNOWN_TYPE at $.types[1].fields[0].type:
  "Unsupported or unknown field type 'inner'."
  expected: "i32 | bool"
  found: "inner"
```

El modelo IR v0.5 (`crates/ir/src/model.rs`, `enum IrType { I32, Bool, Record(String) }`) acepta `Record(String)` como **tipo de parámetro o retorno**, pero el verifier solo permite `i32 | bool` como **tipo de campo de record**. En consecuencia, un record cuyo campo apunta a otro record no es construible.

**Conclusión:** el defecto hipotético **no es alcanzable desde Semantic IR v0.5**. Es una limitación de diseño del tipo, no un bug latente en el emitter. Si una futura IR v0.6+ levanta la restricción (RM-07 / INIT-07-02), este análisis debe revisarse.

**Verificación (`crates/runtime/tests/nested_records.rs`, 1 test):** confirma que el verifier emite `E_UNKNOWN_TYPE` con `expected = "i32 | bool"` y `found = "inner"`. El fixture queda como regresión documental.

## 3. Asignador incremental — observaciones

**Qué se midió:** combinando AN-012 (`cold_start`, `warm_invoke`, `load_only`) y AN-013 (`cached_cold_start`), para el fixture `identity` + `make-point-sum` (program-005).

| Operación | Core WASM (ns) | Component Model Record |
|---|---|---|
| Warm invoke `identity(i32) -> i32` | ~1 990 ns | — |
| Cache-hit cold path (`Resolver::load + invoke`) | ~2 090 ns | — |
| Cold path (`Engine::new + load + invoke`) | ~389 000 ns | — |
| `make-point-sum(x, y) -> record{sum, doubled}` | — | funcional, sin medición dedicada |

**Qué NO se midió (bloqueado por v0.5 o fuera de scope):**

- **E3 anidamiento profundo:** v0.5 no permite records-dentro-de-records (ver §2); el experimento es incompatible con la gramática actual.
- **Bytes exactos de allocación:** Wasmtime 27 no expone un hook público para medir bytes allocados sin interponer `wasmtime::ResourceLimiter`, cuya integración queda fuera del alcance de AN-016 (va a INIT-06-02 según Technical Review). AN-012 Learning ya documentó esta limitación.
- **E1 allocación repetida en una invocación / E2 record grande:** la gramática v0.5 limita severamente la construcción de fixtures de estrés (sin loops en el IR, sin arrays, sin strings). Un fixture con N records intermedios encadenados requeriría N funciones separadas, lo que no mide "varias allocaciones en una invocación".

**Observación cualitativa de los números disponibles:** el costo por invocación caliente (~2 µs para Core WASM identity) está dominado por la construcción del `Store` fresh (ADR-005 baseline operativa). El asignador incremental del guest aparece sumado en ese presupuesto y no es el término dominante.

## 4. Canonical ABI — qué cruza la frontera

Observado experimentalmente con `program-005`:

- **Records retornados:** se copian al host como `wasmtime::component::Val::Record(Vec<(String, Val)>)`. El guest no retiene puntero accesible desde el host.
- **Primitivos retornados:** `wasmtime::component::Val::S32(i32)` por valor. Idéntico al patrón POC.
- **Liberación host-side:** ninguna. El `wasmtime::Store` se destruye al final de cada `invoke` (fresh per invocation, ADR-005 baseline operativa), lo que libera toda memoria lineal del guest.
- **Resources, strings, listas:** no cubiertos (fuera de IR v0.5).

El patrón "copy-on-return + fresh Store per invoke" es consistente con el Canonical ABI estándar y con la decisión conservadora de ADR-005 baseline operativa.

## 5. Síntesis para ADR-005

| Sección ADR-005 | Estado previo | Estado tras AN-016 |
|---|---|---|
| `[open]` Allocator alternativo | Sin evidencia | **[deferred]** a INIT-06-02 con criterio objetivo: antes de proponer un allocator diferente al default `wit-bindgen` hay que medir bytes allocados reales con `ResourceLimiter`. La medición cualitativa de AN-012/013 muestra que el default no es el término dominante. |
| `[open]` Estrategia de liberación para instancias persistentes | Sin evidencia | **[deferred]** a INIT-06-03. El baseline actual (`Store` fresh per invoke) hace la pregunta vacua: no hay instancias persistentes. INIT-06-03 debe decidir si levantar esa restricción, y solo entonces esta decisión aplica. |
| `[open]` Comportamiento ante allocación masiva | Sin fixture ejecutable bajo v0.5 | **[deferred]** a RM-07 / INIT-07-02 cuando la IR permita construcciones repetidas (loops, listas). Hoy la gramática v0.5 limita el experimento. |
| Canonical ABI (sin sección previa) | — | **[chosen]** copy-on-return para records y primitivos; fresh Store per invoke garantiza liberación completa. Documentado aquí §4 y a introducir en ADR-005 §Canonical ABI. |

## 6. Lo que queda abierto como advertencia

- Cuando IR v0.6 habilite records anidados reales, re-ejercer el experimento §2 y validar que `emit_expr`/`record.construct` del compilador no sobrescribe punteros intermedios.
- Cuando `ResourceLimiter` esté integrado (INIT-06-02), re-medir bytes reales por record pequeño y por record "grande" (K campos).
- Mantener la invariante AN-016: cualquier helper que resuelva paths externos debe aceptar un contexto explícito; nunca re-introducir fallbacks `../../`.
