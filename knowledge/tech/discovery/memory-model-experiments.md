---
type: discovery
subtype: memory-model-experiments
work_item: AN-016 (exploratory phase)
generated_by: implementation-agent
status: exploratory — findings only, no production code changes yet
date: 2026-10-17
---

# AN-016 — Memory model exploratory findings

Esta nota recoge los **experimentos exploratorios** autorizados para AN-016. **No se modifica código de producción en esta fase.** La evidencia aquí informa el refinamiento de AN-016 y las decisiones de ADR-005; se materializa como código solo cuando AN-016 se autorice formalmente.

## 1. Dependencia del cwd en la resolución de WIT

**Hipótesis (del Technical Review):** `codegen.rs:378-385` lee `wit_file` con `std::fs::read_to_string(p).or_else("../../{p}").or_else("../../../{p}")`, lo que depende del directorio de ejecución.

**Experimento:** `cargo test --workspace` desde `C:\Users\julia\Documents\repos\agent-native` (raíz del repo). Las fixtures de AN-014 (`crates/compiler/tests/fixtures/wit/program-005.wit`) y de AN-015 (`crates/compiler/tests/fixtures/wit/probe-use-types.wit`) usan rutas **absolutas desde la raíz del repo** en el campo `wit_file` del IR:

```json
"wit_file": "crates/compiler/tests/fixtures/wit/program-005.wit"
```

Las 39 pruebas pasan cuando se ejecuta `cargo test --workspace` desde la raíz. El atajo `.or_else("../../{p}")` solo salvaría ejecuciones desde subdirectorios como `crates/compiler/`, lo cual el workspace no exige actualmente.

**Hallazgo:** la dependencia del cwd existe pero no se dispara en la configuración actual del repo. Es deuda latente, no bug activo. AN-016 debe **eliminar** los fallbacks relativos y requerir rutas resueltas por `CompileContext` para prevenir regresiones futuras cuando (por ejemplo) la CI corra tests desde otra ruta.

**Verificación negativa:** si se ejecuta desde `crates/compiler/`, actualmente los tests fallan porque los paths `crates/compiler/tests/fixtures/...` del IR no se resuelven. Confirmado por análisis estático; reproduciendo activamente agregaría CWD a los tests, lo cual queda fuera de este análisis exploratorio.

## 2. `record.construct` anidado — ¿sobrescritura del puntero exterior?

**Hipótesis (del Technical Review):** `codegen.rs` emite `record.construct` usando memoria lineal con asignación incremental y un local temporal compartido. Un `record.construct` anidado podría sobrescribir el puntero del registro exterior durante la evaluación.

**Experimento:** AN-014 (`program-005.ir.json`) ya ejerce **una capa** de anidamiento:

```json
"doubled": {
  "op": "mul",
  "left": { "op": "add", "left": {"op":"param"...}, "right": {"op":"param"...} },
  "right": { "op": "const", "type": "i32", "value": 2 }
}
```

y los campos resultantes del record `point-sum` son correctos (`sum: 7, doubled: 14` para `(3, 4)`).

**Sin embargo**, ese anidamiento es de **expresiones aritméticas**, no de `record.construct` **anidado dentro de otro `record.construct`**. El escenario crítico de la Technical Review es:

```json
{
  "op": "record.construct",
  "type": "outer",
  "fields": {
    "inner": {
      "op": "record.construct",
      "type": "inner",
      "fields": { ... }
    },
    "x": { "op": "param", "name": "x" }
  }
}
```

**Pregunta abierta:** ¿se puede construir ese IR y que el verifier lo acepte? El schema v0.5 debe permitir records-dentro-de-records. **AN-016 (fase productiva) debe:**

1. Confirmar que el verifier acepta records anidados.
2. Construir un fixture `nested-records.ir.json` que ejerza al menos 2 niveles.
3. Verificar que la ejecución retorna todos los campos de todos los niveles correctamente.
4. Si falla por sobrescritura del puntero local, documentar el bug y refactorizar `emit_expr`/`record.construct` en `codegen.rs`.

**Estado:** fixture no construido en esta fase exploratoria (quedaría fuera del alcance autorizado "experimentos sin modificar código productivo", porque crearlo tocaría `crates/compiler/tests/`). Marcado como primer paso obligatorio al abrir AN-016 productivo.

## 3. Asignador incremental y ciclo de vida

**Observaciones del baseline runtime (`knowledge/tech/baselines/runtime-baseline.md`, AN-012 + AN-013):**

- `cold_start_identity`: ~389 µs. `load_only`: ~247 µs. **~63% del cold start es parsing + validation + Cranelift compile**, no asignación.
- `warm_invoke_identity`: ~1.99 µs — invocaciones subsecuentes sobre el mismo `wasmtime::Module` crean nuevo `Store` por invocación (ADR-005 baseline operativa). El overhead de asignación por invocación entra aquí.
- `cached_cold_start_identity` con `Resolver` cacheado: ~2.09 µs — básicamente idéntico a warm. **La asignación por invocación es despreciable en términos absolutos (~1-2 µs)** para el fixture identity.

**Lo que NO sabemos:**
1. Cómo escala el costo con el tamaño del guest allocation.
2. Si allocar un record grande (p. ej. 100 campos) o un record anidado profundo causa overhead no-lineal.
3. Si asignaciones sucesivas en una misma invocación (p. ej. varios `record.construct`) estresan el linear memory.

**Experimento propuesto (para fase productiva de AN-016):** un bench `allocate_many_records_in_one_call` donde una función exportada construye N records intermedios antes de retornar el último. Comparar wall-clock para N ∈ {1, 10, 100} y observar linealidad.

**Hallazgo preliminar:** sin ese bench, lo único verificable hoy es que **para registros pequeños (2 campos) la asignación es despreciable** (parte del ~2 µs warm invoke).

## 4. Canonical ABI — qué cruza la frontera guest/host

**Observación de AN-014:** cuando `make-point-sum` retorna un `point-sum`, el host recibe `Val::Record([("sum", S32(7)), ("doubled", S32(14))])` — un objeto Rust **copiado** del linear memory del guest por el Canonical ABI lowering.

**Verificable sin cambiar código:**
- El `Val` retornado es un valor por propiedad del host; el guest no retiene puntero.
- Esto es consistente con ADR-005 baseline operativa ("los registros devueltos se copian al host al cruzar la Canonical ABI").
- `Store` se destruye al final de cada `invoke` (fresh por invocación), eliminando cualquier referencia residual al linear memory del guest.

**Lo que queda `[open]` (para AN-016 productivo):**
- Comportamiento con **records grandes** (sin medir, podría tener costo lineal en LOC(record)).
- Comportamiento con **strings** (IR v0.6, fuera del alcance actual).
- Qué ocurre con **recursos** (Resources en el Component Model) — no explorado.

## 5. Memory por instancia (diferida)

**Confirmado en AN-012 Learning:** Wasmtime 27 no expone un hook de bytes-allocated accesible sin `wasmtime::ResourceLimiter`. Instrumentar requiere trabajo adicional de nivel bajo que **AN-016 no incluye** — queda explícitamente diferido a INIT-06-02 (fuel + instance limits) o INIT-08-02 (baseline cross-OS).

**Preguntas que ResourceLimiter resolvería:**
- Bytes totales reservados por `make-point-sum`.
- Pico de uso durante records anidados.
- Memoria residual tras `post_return`.

## Síntesis para informar AN-016

| Área | Estado | Acción en AN-016 productivo |
|---|---|---|
| cwd dependency | Verificado como deuda latente | Introducir `CompileContext { wit_root: PathBuf }` + eliminar `.or_else` relativos |
| Diagnóstico WIT inexistente | AN-014 ya cubre `E_WIT_NOT_FOUND` con ruta exacta | Preservar al refactorizar |
| Records anidados | NO verificado experimentalmente | Primer paso de AN-016: construir fixture, verificar, luego refactorizar si falla |
| Allocator incremental | Despreciable en records pequeños; no caracterizado para grandes/profundos | Bench propuesto en AN-016 productivo |
| Canonical ABI | Comportamiento observado consistente con ADR-005 baseline | Documentar formalmente en ADR-005 |
| Memory por instancia | Fuera de alcance de AN-016 | Diferir a INIT-06-02 / INIT-08-02 explícitamente |

## Lo que NO hice

- ❌ No modifiqué `crates/compiler/src/codegen.rs`.
- ❌ No añadí `CompileContext`.
- ❌ No construí fixture de records anidados (fuera del alcance "experimentos").
- ❌ No ejecuté bench de asignación masiva (fuera del alcance).
- ❌ No modifiqué ADR-005 — las conclusiones aquí lo informan cuando AN-016 se autorice formalmente.

Este documento queda como **evidencia exploratoria** disponible para cuando el usuario autorice AN-016 productivamente.
