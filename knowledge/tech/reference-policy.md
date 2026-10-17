---
type: reference-policy
project_state: ai-assisted
generated_by: implementation-agent
template_version: 1
work_item: AN-001
---

> Idioma del proyecto: **español**. Mantener en inglés el código, los nombres de archivo, los comandos y las claves de configuración.

# Política de referencia — `_reference/poc-agent-native/`

Este documento es la política canónica sobre el uso de la POC de referencia dentro del repositorio `Kaddo-kdd/agent-native`. Está vinculado al Work Item [AN-001](../delivery/work-items/in-progress/AN-001-govern-reference-poc.md) y a la iniciativa INIT-01-04 del [roadmap](../delivery/roadmap.md).

## Alcance y origen

- Repositorio producto: [`Kaddo-kdd/agent-native`](https://github.com/Kaddo-kdd/agent-native).
- POC de referencia: [`Trycatch-tv/poc-agent-native`](https://github.com/Trycatch-tv/poc-agent-native).
- Commit congelado de referencia: `26b6a18e2f4a50086d5ee604051154d9ea77d768`.
- Ruta local esperada: `_reference/poc-agent-native/`.

La POC no es un submódulo del repositorio nuevo. Se obtiene mediante `git clone` directo bajo `_reference/`, nunca mediante `git submodule add`.

## Reglas

### 1. Solo lectura

`_reference/poc-agent-native/` es **de solo lectura** dentro del flujo de trabajo del producto nuevo. Ningún Work Item de `Kaddo-kdd/agent-native` puede modificar, renombrar, reorganizar ni borrar archivos bajo `_reference/`. Si un cambio en la POC fuera necesario, debe hacerse en su propio repositorio origen por separado, nunca como parte de un cambio del producto nuevo.

### 2. Exclusión de Git

El `.gitignore` raíz incluye la regla `/_reference/`. Esta regla impide que los archivos de la POC entren al índice de Git del repositorio nuevo. Ni manualmente ni mediante `git add -f` debe forzarse la inclusión de archivos de `_reference/` en commits del producto.

### 3. Prohibición de dependencias técnicas

Ningún `Cargo.toml`, `cargo workspace`, `Cargo.lock`, script de build, archivo de CI, módulo Rust o configuración del producto nuevo puede contener:

- Dependencias por ruta hacia la POC, por ejemplo `path = "../_reference/poc-agent-native/..."` o `path = "_reference/..."`.
- Referencias a crates publicados **únicamente** dentro de la POC.
- Rutas de inclusión (`include!`, `mod file;`, scripts generadores) que lean código fuente desde `_reference/`.
- Enlaces a artefactos binarios, WASM Components, fixtures WIT o datos de prueba localizados exclusivamente bajo `_reference/`.

Si un fixture, contrato WIT o caso de prueba de la POC debe reutilizarse, se migra como copia explícita dentro de `tests/`, `examples/`, `wit/` o la ubicación equivalente del producto nuevo, dejando constancia en el Work Item correspondiente.

### 4. Consulta autorizada por agentes

Los agentes (humanos, IDEs, LLMs, scripts) **pueden** leer archivos bajo `_reference/` para:

- Recuperar comportamiento validado por la POC.
- Consultar contratos WIT, Semantic IR o manifiestos.
- Analizar reportes históricos (`BENCHMARK_REPORT_*.md`, `RELEASE_CANDIDATE_*.md`).
- Catalogar pruebas y fixtures como fuente de la **matriz de extracción** (INIT-01-02).

Pero la implementación resultante debe escribirse exclusivamente dentro de los módulos del producto nuevo (`crates/`, `wit/`, `tests/`, `examples/`, `docs/`, `knowledge/`). El agente debe citar el archivo consultado en el Work Item cuando la decisión de implementación se apoye en él.

### 5. Interoperabilidad conservada

Esta política no impide que el runtime del producto nuevo cargue, en tiempo de ejecución, componentes WIT-compatibles **generados** por la POC u otras herramientas externas, siempre que ese escenario forme parte de un Work Item con criterios de aceptación propios. La prohibición se aplica a dependencias de **construcción**, no al consumo runtime de artefactos externos.

### 6. `.gitignore` no es suficiente por sí solo

`.gitignore` evita que archivos entren al índice de Git, pero **no** impide que un IDE, un agente de IA o un script del sistema de archivos lea, escriba o reemplace archivos bajo `_reference/`. Esta política suministra las instrucciones explícitas de trabajo que sí lo impiden. Cualquier automatización que opere sobre el repositorio debe respetarla.

### 7. Agentes externos

Esta política aplica a toda herramienta que opere sobre el repositorio: humanos, IDEs (p. ej. VS Code, JetBrains, Cursor), agentes de IA (p. ej. Claude Code, GitHub Copilot, Cursor Agent) y pipelines automatizados. Si una herramienta no la respeta, su configuración debe ajustarse — no la política.

## Verificación

### Comandos y resultados esperados

**Verificación inmediata (siempre aplicable):**

```bash
# 1. La regla existe
git check-ignore -v _reference/poc-agent-native/Cargo.toml
# → Resultado esperado: una línea con
#   .gitignore:<n>:/_reference/ _reference/poc-agent-native/Cargo.toml

# 2. Ningún archivo de la POC entra al índice
git ls-files _reference/
# → Resultado esperado: salida vacía

# 3. git status no reporta archivos de la POC como pendientes
git status --short
# → Resultado esperado: sin líneas bajo _reference/
```

**Verificación diferida (una vez exista workspace Rust por INIT-01-01):**

```bash
# 4. cargo metadata no lista paquetes de la POC
cargo metadata --format-version=1 --no-deps | \
  jq -r '.packages[].manifest_path' | \
  grep -i "_reference" || echo "OK: sin paquetes bajo _reference/"
# → Resultado esperado: "OK: sin paquetes bajo _reference/"

# 5. cargo check no intenta compilar fuentes de la POC
cargo check --workspace 2>&1 | grep -i "_reference" && exit 1 || true
# → Resultado esperado: sin coincidencias
```

### Qué hacer si una verificación falla

- **Falla 1 (`git check-ignore` sin coincidencia):** restaurar la regla `/_reference/` en `.gitignore`.
- **Falla 2 (`git ls-files _reference/` no vacío):** `git rm --cached` los archivos listados; abrir un WI de corrección si no es evidente cómo entraron.
- **Falla 3 (`git status` reporta cambios bajo `_reference/`):** revisar si alguien modificó la POC desde el repositorio nuevo (prohibido por la regla 1). Revertir y abrir un WI de incidente.
- **Falla 4 o 5 (`cargo metadata`/`cargo check`):** eliminar la dependencia hacia `_reference/` del `Cargo.toml` ofensor; abrir un WI de corrección si fue introducida por un agente automatizado.

## Procedimiento de clonación local

```bash
# Desde la raíz de agent-native
git clone https://github.com/Trycatch-tv/poc-agent-native _reference/poc-agent-native
cd _reference/poc-agent-native
git checkout 26b6a18e2f4a50086d5ee604051154d9ea77d768
cd -
```

Si existe una carpeta `.git` dentro de `_reference/poc-agent-native/`, es parte de la clonación de la POC y es correcta. No se añade como submódulo del repositorio nuevo.

## Vigencia

Esta política está vigente mientras `_reference/poc-agent-native/` sea la fuente histórica canónica del producto. Se revisa si:

- La POC se retira del repositorio porque la extracción está completa (criterio de salida de RM-01 ampliado).
- Se decide ingerir otra POC o repositorio histórico adicional.
- Un ADR futuro modifica las reglas de interoperabilidad o composición multi-origen (ver objetivo arquitectónico diferido en el roadmap).

Cualquier cambio a esta política se propone mediante un Work Item explícito.

## Referencias

- Work Item: [AN-001](../delivery/work-items/in-progress/AN-001-govern-reference-poc.md)
- Iniciativa roadmap: INIT-01-04 (bloque RM-01) en [roadmap.md](../delivery/roadmap.md)
- Mapa del codebase: [codebase.md](codebase.md)
- Estado técnico: [current-state.md](current-state.md)
