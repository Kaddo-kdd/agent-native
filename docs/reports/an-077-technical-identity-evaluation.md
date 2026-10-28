# AN-077 — Evaluación de Identidad Técnica, Disponibilidad de Nombres y Estrategia de Migración

**Iniciativa:** INI-009 — Public Release, Technical Identity & Developer Adoption  
**Work Item:** AN-077 (Spike, P0, K3)  
**Fecha:** 2026-10-28  
**Autor:** Antigravity / Julián Darío Luna Patiño  
**Estado:** Aprobado por decisión humana (ADR-008 aceptado)  

---

## 1. Resumen Ejecutivo y Contexto

Wasma completó exitosamente su núcleo técnico a través de 67 Work Items y 8 iniciativas (INI-001 a INI-008). Su presencia comercial y pública ya opera bajo la marca **Wasma** (`wasma.org`), pero las interfaces técnicas internas y externas continúan utilizando la denominación de desarrollo inicial **Agent Native**:
- Binario CLI: `agent-native`
- Paquete Cargo de la CLI: `agent-native`
- Crates de biblioteca: `agent-native-ir`, `agent-native-compiler`, `agent-native-runtime`
- Repositorio GitHub: `https://github.com/Kaddo-kdd/agent-native`
- Variables de entorno: `AGENT_NATIVE_LLM_*`
- Configuración de Kaddo: `.kaddo/config.yml` (`name: agent-native`)

De cara a la primera versión pública **v0.1.0**, esta evaluación spike investiga la disponibilidad de nombres, audita exhaustivamente el repositorio, compara las alternativas de migración y formula una propuesta técnica con garantías de compatibilidad, rollback y preservación de la trazabilidad histórica.

La propuesta ha sido formalmente revisada y **aprobada por decisión humana**, adoptando la identidad Wasma y formalizándose en el **ADR-008**.

---

## 2. Inventario Técnico de Referencias y Estructura Real

Se realizó una auditoría automatizada en el workspace (`git grep` exhaustivo sobre nombres con guion y guion bajo: `agent-native` y `agent_native`).

### Resumen cuantitativo por superficies

| Superficie | Archivos afectados | Ocurrencias | Tratamiento aprobado |
|---|---|---|---|
| **Manifiestos Cargo y Lockfile** (`Cargo.toml`, `Cargo.lock`) | 2 | 11 | Migrar a nombres Wasma (`wasma-cli`, `wasma-ir`, etc.) |
| **Manifiestos de Crates** (`crates/*/Cargo.toml`) | 4 | 13 | Migrar `package.name`, dependencias internas relativas y metadata |
| **Código Fuente Rust** (`crates/*/src/**/*.rs`) | 40 | 192 | Actualizar declaraciones `use`, paths de módulos y cadenas internas |
| **Variables de Entorno y Rutas Temporales** | 3 | 11 | Introducir `WASMA_LLM_*` con fallback seguro a `AGENT_NATIVE_LLM_*` |
| **Workflows de CI/CD** (`.github/workflows/*.yml`) | 4 | 16 | Actualizar nombres de paquetes en invocaciones `cargo`, artefactos y binarios |
| **Scripts de Automatización y Benchmarks** (`scripts/*.py`) | 2 | 6 | Actualizar parámetros `-p` de Cargo |
| **Portal Web y Documentación** (`website/`, `docs/`) | 24 | ~964 | Actualizar guías, tutoriales interactivos, comandos CLI y enlaces canónicos |
| **Ejemplos de Referencia** (`examples/`) | 12 | ~57 | Actualizar dependencias de crates y comandos en READMEs |
| **Configuración Kaddo** (`.kaddo/config.yml`) | 1 | 1 | Preservar trazabilidad e identificador local del proyecto |
| **Histórico Inmutable** (WIs AN-001..AN-067, ADR-001..ADR-007) | 51 | ~250 | **CONSERVAR INMUTABLE** por integridad histórica y auditoría |

### Detalle de estructura real de dependencias Cargo

El workspace actual **no utiliza `[workspace.dependencies]`**. Cada crate declara sus dependencias internas mediante rutas relativas directas:
- `crates/compiler/Cargo.toml`: `agent-native-ir = { path = "../ir" }`
- `crates/runtime/Cargo.toml`: `agent-native-ir = { path = "../ir" }`, dev-dependency `agent-native-compiler = { path = "../compiler" }`
- `crates/cli/Cargo.toml`: `agent-native-ir = { path = "../ir" }`, `agent-native-compiler = { path = "../compiler" }`, `agent-native-runtime = { path = "../runtime" }`

En **AN-078**, las dependencias internas se actualizarán a sus nuevos nombres locales (`wasma-ir = { path = "../ir" }`, etc.).  
Posteriormente, en **AN-068** y **AN-070**, cuando se prepare la publicación a crates.io, se añadirá la clave `version = "0.1.0"` (`path + version`) según los requisitos normativos de Cargo para crates publicados.

---

## 3. Matriz de Disponibilidad de Nombres y Colisiones

Se realizaron consultas sobre registros externos (`crates.io` y `GitHub`):

| Nombre / Variante | Registro | Estado | Titular / Situación | Impacto y Resolución Aprobada |
|---|---|---|---|---|
| **`wasma`** | `crates.io` | **OCUPADO (Colisión)** | Azencorporation (*Windows Assignment System Monitoring Architecture*) | **No se puede registrar el paquete Cargo `wasma`**. No obstante, **el binario ejecutable SÍ puede llamarse `wasma`** configurando `name = "wasma-cli"` con `[[bin]] name = "wasma"`. |
| **`wasma-cli`** | `crates.io` | **DISPONIBLE** | Libre | **Nombre aprobado para el crate de la CLI**. Permite `cargo install wasma-cli`, instalando únicamente el binario `wasma`. |
| **`wasma-runtime`** | `crates.io` | **DISPONIBLE** | Libre | Crate del runtime independiente. Sin colisiones registradas. |
| **`wasma-compiler`** | `crates.io` | **DISPONIBLE** | Libre | Crate del compilador Semantic IR → WASM. Sin colisiones. |
| **`wasma-ir`** | `crates.io` | **DISPONIBLE** | Libre | Crate del modelo y verificador IR. Sin colisiones. |
| **`Kaddo-kdd/wasma`** | `GitHub` | **DISPONIBLE** | Organización Kaddo-kdd | Nombre libre dentro de la organización titular. |
| **`stg-tud/wasma`** | `GitHub` | Proyecto externo | TU Darmstadt (*WebAssembly Static Analysis Framework* en Go) | Sin conflicto de namespace; organizaciones independientes. |

### Consideraciones sobre exclusividad y re-verificación
1. **Exclusividad en el sistema operativo:**  
   Nombrar el paquete `wasma-cli` resuelve la colisión en el índice de `crates.io`. Sin embargo, esto no garantiza exclusividad del comando ejecutable `wasma` a nivel de sistema operativo si el usuario tiene instaladas otras herramientas en su `PATH`. Se documentará claramente en la guía de instalación la verificación del binario (`wasma --version`).
2. **Re-verificación obligatoria antes de publicar:**  
   Antes de ejecutar la publicación en `crates.io` en AN-076, se volverá a comprobar la disponibilidad de `wasma-cli`, `wasma-runtime`, `wasma-compiler` y `wasma-ir` para prevenir carreras de registro por terceros.

---

## 4. Comparación de Estrategias y Decisión Aprobada

| Criterio | Alternativa 1: Renombramiento Total Inmediato | Alternativa 2: Solo Fachada Pública | Alternativa 3: Migración Integral Faseada (Aprobada) |
|---|---|---|---|
| **Descripción** | Cambiar código, crates y repositorio simultáneamente en un solo paso. | Cambiar el binario a `wasma`, pero dejar crates y repo como `agent-native`. | Separar en AN-078 (código y workspace local) y AN-079 (repositorio e integraciones). |
| **Coherencia Técnica** | Alta, pero propensa a fallos no aislados. | Muy baja. Fuga de abstracción en `Cargo.lock` y crates de biblioteca. | **Máxima**. Todo el código y crates quedan unificados limpiamente. |
| **Riesgo Operativo** | Alto (puede romper CI, hosting y build a la vez). | Bajo a corto plazo, insostenible a mediano plazo. | **Bajo y controlado**. Cada fase tiene su suite de pruebas y compuerta de verificación. |
| **Compatibilidad Ejecutable** | Ruptura abrupta sin transición. | Confusión conceptual permanente. | **Instalación estándar proporciona exclusivamente `wasma`**. Alias opcional bajo feature flag no predeterminado. |
| **Facilidad de Rollback** | Compleja (diff gigantesco no desacoplado). | N/A (no soluciona la deuda). | **Limpia y determinista** (aislada por commit y fase, sin `git reset --hard`). |

---

## 5. Decisiones Técnicas Específicas

### 5.1. Compatibilidad del ejecutable anterior
- La instalación estándar de `wasma-cli` **proporcionará exclusivamente el ejecutable `wasma`** (o `wasma.exe` en Windows). No se instalará `agent-native` por defecto para evitar contaminar el sistema del usuario con binarios obsoletos.
- Si en entornos corporativos o scripts legados se requiere compatibilidad, se habilitará un alias mediante una feature flag explícita y deshabilitada por defecto en Cargo:
  `--features legacy-alias`
- **Política de deprecación:** Dicho alias emitirá un warning visible en `stderr`:
  `[WARN] El comando 'agent-native' está obsoleto y será retirado en Wasma v0.2.0. Use 'wasma' en su lugar.`

### 5.2. Variables de entorno y seguridad
- **Configuración oficial:** Se aprueba el prefijo `WASMA_LLM_*`:
  - `WASMA_LLM_KEY`
  - `WASMA_LLM_PROVIDER`
  - `WASMA_LLM_MODEL`
  - `WASMA_LLM_ENDPOINT`
  - `WASMA_LLM_TIMEOUT`
- **Precedencia:** `WASMA_LLM_*` prevalece estrictamente sobre `AGENT_NATIVE_LLM_*`.
- **Tratamiento de valores vacíos:** Una variable presente pero vacía (`""`) o compuesta sólo de espacios en blanco se trata como no definida, cayendo en el fallback o en el valor por defecto.
- **Protección de credenciales en avisos de deprecación:**  
  Si se recurre al fallback de `AGENT_NATIVE_LLM_*`, la CLI emitirá en `stderr` un mensaje de advertencia mencionando **únicamente el nombre de la variable**, sin imprimir bajo ninguna circunstancia su valor o secreto:
  `[WARN] Se detectó la variable obsoleta AGENT_NATIVE_LLM_KEY. Migre a WASMA_LLM_KEY.`

### 5.3. Garantías de compatibilidad e invariantes arquitectónicas
- **Semantic IR v0.5 / v0.6:** Formato JSON/YAML agnóstico de marca; sin cambios en especificación ni esquemas.
- **Contratos WIT y Canonical ABI:** Preservados sin alteración; los bindings de Component Model mantienen compatibilidad binaria byte a byte.
- **Invariantes del Runtime (ADR-005):** `Store` e instancia fresca por ejecución, aislamiento de tenants, cuotas de fuel y límites de memoria estrictamente intactos.
- **Workers experimentales:** Deshabilitados por defecto bajo flag experimental.

---

## 6. Clasificación de Referencias: Migración vs. Preservación Histórica

### Superficies que DEBEN migrarse (en AN-078 y AN-079)
- `Cargo.toml` raíz y `Cargo.lock`.
- `crates/cli/Cargo.toml` → `wasma-cli` (bin `wasma`, lib `wasma_cli`).
- `crates/ir/Cargo.toml` → `wasma-ir`.
- `crates/compiler/Cargo.toml` → `wasma-compiler`.
- `crates/runtime/Cargo.toml` → `wasma-runtime`.
- Código Rust en `crates/*/src/`: imports `use wasma_ir::*`, `use wasma_compiler::*`, `use wasma_runtime::*`, `use wasma_cli::*`.
- GitHub Actions: `.github/workflows/{ci, release, bench, website}.yml`.
- Scripts auxiliares: `scripts/bench_collect.py`, `scripts/worker_bench_collect.py`.
- Validador y contenido del portal web: `website/scripts/test-docs.mjs`, páginas de documentación y tutoriales.
- Documentación activa: `README.md`, `docs/`.

### Referencias que DEBEN conservarse intactas (Trazabilidad Histórica)
- **Work Items completados:** `knowledge/delivery/work-items/completed/AN-001` a `AN-067`.
- **ADRs previos aceptados:** `ADR-001` a `ADR-007` en `knowledge/tech/decisions/`.
- **Reportes históricos de benchmarks:** `docs/benchmarks/history.md`, datos en `docs/benchmarks/publication.json`.

---

## 7. Plan de Gestión de Riesgos y Rollback Seguro

| Riesgo | Probabilidad | Impacto | Mitigación Aprobada |
|---|---|---|---|
| **R1: Colisión de nombres en crates.io** | Alta (Confirmada para `wasma`) | Alto | Resuelto: crate `wasma-cli` y librerías `wasma-{ir,compiler,runtime}`. Re-verificar antes de publicar. |
| **R2: Colisión de ejecutable en SO** | Baja | Medio | Documentar comprobación de versión y PATH en guía de instalación. |
| **R3: Rotura del validador de documentación** | Media | Medio | `website/scripts/test-docs.mjs` se actualiza simultáneamente en AN-078 para evaluar `wasma`. |
| **R4: Interrupción de CI** | Baja | Alto | Validar workspace completo en local con `cargo test --workspace` antes de commit. |
| **R5: Dependencias externas en GitHub** | Media | Alto | En AN-079, no confiar ciegamente en redirecciones; auditar GitHub Actions, Netlify, badges y webhooks. |

### Estrategia de Rollback Seguro (Sin `git reset --hard`)
- **Aislamiento en rama de trabajo:** Toda la migración técnica de AN-078 se desarrolla en una rama específica o sobre un working tree limpio verificado.
- **Reversión mediante commits trazables:** En caso de incidencia, la recuperación se realiza mediante `git revert <commit-sha>`, registrando la causa y conservando la auditoría de cambios sin descartar modificaciones locales ni destruir el historial.
- **Control de estado en AN-079:** Si el cambio de repositorio en GitHub introduce fallos con Netlify o herramientas externas, el nombre remoto se restablece desde la interfaz administrativa de GitHub sin pérdida de datos ni commits.

---

## 8. Plan Secuencial de Ejecución para AN-078 y AN-079

### Fase A: AN-078 — Migración Técnica de Código y Workspace
1. **Paso 1: Manifiestos de Cargo y Dependencias Internas:**
   - Actualizar `crates/ir/Cargo.toml` (`wasma-ir`).
   - Actualizar `crates/compiler/Cargo.toml` (`wasma-compiler`, dependencias relativas).
   - Actualizar `crates/runtime/Cargo.toml` (`wasma-runtime`, dependencias relativas).
   - Actualizar `crates/cli/Cargo.toml` (`wasma-cli`, bin `wasma`, lib `wasma_cli`, dependencias relativas, feature `legacy-alias` deshabilitada por defecto).
   - Actualizar `Cargo.toml` raíz (metadata del workspace).
   - Regenerar `Cargo.lock` mediante `cargo check --workspace`.
2. **Paso 2: Código Rust:**
   - Sustituir imports `agent_native_ir` → `wasma_ir`.
   - Sustituir imports `agent_native_compiler` → `wasma_compiler`.
   - Sustituir imports `agent_native_runtime` → `wasma_runtime`.
   - Sustituir imports `agent_native` → `wasma_cli`.
   - Implementar resolución segura de `WASMA_LLM_*` con fallback a `AGENT_NATIVE_LLM_*` y sanitización de credenciales en logs de advertencia.
3. **Paso 3: Pruebas y Validación Real:**
   - Ejecutar `cargo fmt --check`, `cargo clippy`, `cargo test --workspace`.
   - Ejecutar benchmarks reales existentes: `cargo test --locked -p wasma-runtime --example bench_latency` y `python scripts/bench_collect.py --help`.
4. **Paso 4: CI/CD y Documentación de Desarrollo:**
   - Actualizar llamadas en `.github/workflows/{ci, release, bench, website}.yml`.
   - Actualizar `website/scripts/test-docs.mjs` y contenido de tutoriales.
   - Validar suite web con `pnpm --dir website test`.

### Fase B: AN-079 — Migración de Repositorio GitHub y Continuidad de Integraciones
1. **Paso 1: Autorización Humana Explícita** para la operación remota en GitHub.
2. **Paso 2: Renombrar repositorio** `Kaddo-kdd/agent-native` → `Kaddo-kdd/wasma`.
3. **Paso 3: Actualizar remotos locales:** `git remote set-url origin https://github.com/Kaddo-kdd/wasma.git`.
4. **Paso 4: Actualizar `Cargo.toml` raíz:** `repository = "https://github.com/Kaddo-kdd/wasma"`.
5. **Paso 5: Auditoría y Verificación de Integraciones:**
   - Verificar conexión de hosting en Netlify para `wasma.org`.
   - Verificar webhooks y GitHub Actions en la nueva URL canónica.
   - Comprobar que `git clone` y enlaces canónicos redirigen y resuelven correctamente.

---

## 9. Conclusión y Registro de Decisión

La estrategia aprobada resuelve de forma rigurosa la colisión en `crates.io`, garantiza una experiencia de usuario limpia con el comando `wasma`, y establece salvaguardas de seguridad, precedencia y reversibilidad para la ejecución de **AN-078** y **AN-079**.
