# Guía Rápida de la CLI de Agent-Native

`agent-native` es la interfaz binaria unificada para compilar, verificar, probar, ejecutar y publicar especificaciones y componentes WebAssembly Component Model sin requerir dependencias externas de red o LLM en tiempo de ejecución.

---

## 1. Subcomandos Canónicos

La CLI soporta 5 subcomandos principales:

1. **`verify`**: Verifica estáticamente la validez de una especificación narrativa (`.md`) o de un documento Semantic IR (`.json`).
2. **`build`**: Compila una especificación narrativa o Semantic IR a un componente binario WebAssembly (`.wasm`) con cálculo de hash SHA-256.
3. **`run`**: Carga un componente `.wasm` y ejecuta una función exportada mediante el runtime independiente `agent-native-runtime`, con verificación criptográfica y límites de ejecución configurables.
4. **`test`**: Ejecuta suites de pruebas de aceptación en formato YAML contra un componente `.wasm`.
5. **`publish`**: Empaqueta y publica atómicamente la especificación y sus componentes en el almacén de versiones local con manifiesto, firma y trazabilidad.

---

## 2. Modos de Salida y Códigos de Salida

### Formatos de Salida (`--format`)

- `--format human` (por defecto): Salida amigable formateada para terminal.
- `--format json`: Salida estructurada pura por `stdout` (100% parseable por herramientas de CI/CD, sin texto intercalado).

### Códigos de Salida

- `0` (`Success`): Operación completada satisfactoriamente.
- `1` (`DomainFailure`): Error de dominio (fallo de verificación semántica, error de compilación, aserción de suite de pruebas no superada, o función de runtime con error tipado).
- `2` (`UsageOrIoError`): Error de argumentos CLI, bandera no reconocida, archivo inexistente o fallo de lectura/escritura en disco.

---

## 3. Modos de Generación: Heurística, Fixtures y LLM Real

Los subcomandos `build` y `publish` aceptan la bandera `--mode <synthesizer|fixtures|real>`:

| Modo | Estado Operativo | Comportamiento | Limitaciones Semánticas / Requisitos |
| :--- | :--- | :--- | :--- |
| **`fixtures`** | Operativo | Carga artefactos y esquemas predefinidos desde disco/memoria. | Limitado exclusivamente a los fixtures registrados para pruebas. |
| **`synthesizer`** | Operativo | Sintetizador heurístico determinista basado en reglas estructurales (AST a Semantic IR v0.5). | **No es IA**: genera transformaciones mecánicas elementales (e.g., `param * 2`, inicialización de records y defaults primitivos). No interpreta lenguaje natural libre. |
| **`real`** | Operativo | Integración HTTP con proveedores LLM externos (`HttpLlmProvider`, AN-046). Soporta Anthropic (`messages`) y OpenAI (`chat/completions`). | Requiere `AGENT_NATIVE_LLM_KEY`. Endpoint y modelo configurables mediante `AGENT_NATIVE_LLM_ENDPOINT` y `AGENT_NATIVE_LLM_MODEL`. Valida el IR mediante `verify_ir()`. Cero fallback silencioso hacia heurística (ADR-004). |

---

## 4. Cuotas y Límites de Ejecución en `agent-native run` (AN-047)

El subcomando `run` permite configurar límites deterministas de recursos computacionales antes de instanciar y ejecutar el componente:

```bash
agent-native run \
  --component ./dist/calculator.wasm \
  --function calculate \
  --fuel 100000 \
  --max-memory-bytes 16777216 \
  10 20
```

### Opciones de Recursos Disponibles

- `--fuel <U64>`: Presupuesto máximo de unidades de fuel computacional para la invocación. Si se agota, termina con error tipado `Computational fuel exhausted` (código de salida 1).
- `--max-memory-bytes <USIZE>`: Límite máximo de memoria lineal WebAssembly en bytes (ej. `16777216` para 16 MiB). Si el componente requiere más memoria inicial o intenta crecer más allá del límite, termina con `Memory limit exceeded` (código de salida 1).
- `--max-table-elements <U32>`: Límite máximo de elementos en tablas WebAssembly.

### Salida JSON Estructurada con Métricas de Fuel

Cuando se ejecuta con `--format=json` y `--fuel`:
```json
{
  "success": true,
  "component_path": "./dist/calculator.wasm",
  "function_name": "calculate",
  "results": [30],
  "results_formatted": ["S32(30)"],
  "fuel_consumed": 1250
}
```

### Distinción Técnica entre Fuel, Tiempo Real y Memoria Guest

1. **Fuel Computacional vs. Tiempo Real:**
   - **Fuel** mide instrucciones abstractas y bloques básicos ejecutados por la máquina virtual WebAssembly. Es **100% determinista**: el mismo componente con los mismos argumentos consumirá idénticas unidades de fuel en cualquier máquina.
   - El **tiempo de pared (wall-clock CPU time)** depende de la carga, frecuencia y planificación del sistema operativo anfitrión. No debe emplearse como cuota determinista.
2. **Memoria Guest vs. Memoria del Proceso Anfitrión:**
   - `--max-memory-bytes` restringe exclusivamente las páginas de memoria lineal (`memory.grow`) del componente guest mediante `ResourceLimiter` de Wasmtime.
3. **Invocaciones sin Límites (`unmetered`):**
   - Ejecutar `agent-native run` sin flags de límites es un modo de desarrollo permisivo (`unmetered`), que omite el campo `fuel_consumed`. **No equivale a ejecución segura para código de componentes no confiables**.

---

## 5. Verificación de Integridad y Firmas en Carga (AN-041, AN-042)

El subcomando `run` permite certificar la procedencia e inalterabilidad del artefacto antes de inicializar el runtime:

```bash
# Verificación mediante hash explícito o manifiesto
agent-native run -c component.wasm -f run --verify --expected-sha256 <HEX>

# Verificación criptográfica Minisign Ed25519 (100% offline)
agent-native run -c component.wasm -f run --verify --signature component.wasm.minisig --public-key <PUBKEY>
```

---

## 6. Ejemplo Reproducible Paso a Paso

A continuación se muestra un flujo completo end-to-end usando una única especificación Markdown.

### 6.1. Crear la especificación (`spec.md`)

```markdown
# Sistema de Perfil de Usuario

## 1. Visión general del producto
Servicio para gestión de perfiles de usuario.
1. **Administrador de Perfil (`profile-manager`):** Maneja puntuaciones y perfiles.

## 2. Entidades de Dominio
### Puntuación de Usuario (`UserScore`)
- `score_id`: Identificador de la puntuación.
- `subtotal_cents`: Monto neto base.
- `total_cents`: Monto total acumulado.

## 3. Componente A: Perfil (`profile-manager`)
### Reglas
- **`PROF-001` (Cálculo de puntaje):** Asigna puntos base según actividad.

## 4. Contratos de Interfaz (WIT)
### Interfaz Exportada por `profile-manager`
```wit
interface ops {
    calculate-score: func(subtotal-cents: s32) -> s32;
}
```
```

### 6.2. Crear la suite de pruebas (`suite.yaml`)

```yaml
suite: profile-pipeline-suite
spec_version: 1
cases:
  - id: PROF-CASE-001
    requirement: PROF-001
    function: an:synthesized/ops@0.1.0#calculate-score
    args: [21]
    expected: 42
```

### 6.3. Ejecución de los comandos

```bash
# 1. Verificar la especificación
agent-native verify spec.md

# 2. Compilar a componente WebAssembly
agent-native build spec.md -o profile-manager.wasm --mode synthesizer

# 3. Ejecutar función con límites de fuel y memoria
agent-native run -c profile-manager.wasm -f calculate-score 25 --fuel 100000 --max-memory-bytes 16777216
# Salida esperada:
# Function 'calculate-score' returned 1 value(s):
#   [0]: S32(50)
# Fuel consumed: 1250 units

# 4. Validar contra la suite de aceptación
agent-native test -s suite.yaml -c profile-manager.wasm

# 5. Publicar en el almacén de distribución local
agent-native publish -s spec.md -d ./dist --source-wi AN-028 --mode synthesizer
```
