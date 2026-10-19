# Guía Rápida de la CLI de Agent-Native

`agent-native` es la interfaz binaria unificada para compilar, verificar, probar, ejecutar y publicar especificaciones y componentes WebAssembly Component Model sin requerir dependencias externas de red o LLM.

---

## 1. Subcomandos Canónicos

La CLI soporta 5 subcomandos principales:

1. **`verify`**: Verifica estáticamente la validez de una especificación narrativa (`.md`) o de un documento Semantic IR (`.json`).
2. **`build`**: Compila una especificación narrativa o Semantic IR a un componente binario WebAssembly (`.wasm`) con cálculo de hash SHA-256.
3. **`run`**: Carga un componente `.wasm` y ejecuta una función exportada mediante el runtime independiente `agent-native-runtime`.
4. **`test`**: Ejecuta suites de pruebas de aceptación en formato YAML contra un componente `.wasm`.
5. **`publish`**: Empaqueta y publica atómicamente la especificación y sus componentes en el almacén de versiones local con manifiesto y trazabilidad.

---

## 2. Modos de Salida y Códigos de Salida

### Formatos de Salida (`--format`)

- `--format human` (por defecto): Salida amigable formateada para terminal.
- `--format json`: Salida estructurada pura por `stdout` (100% parseable por herramientas de CI/CD, sin texto intercalado).

### Códigos de Salida

- `0` (`Success`): Operación completada satisfactoriamente.
- `1` (`DomainFailure`): Error de dominio (fallo de verificación semántica, error de compilación, aserción de suite de pruebas no superada, o función de runtime con error).
- `2` (`UsageOrIoError`): Error de argumentos CLI, bandera no reconocida, archivo inexistente o fallo de lectura/escritura en disco.

---

## 3. Modos de Generación: Heurística y Limitaciones

El subcomando `build` y `publish` aceptan la bandera `--mode <synthesizer|fixtures|real>`:

| Modo | Estado Operativo | Comportamiento | Limitaciones Semánticas |
| :--- | :--- | :--- | :--- |
| **`fixtures`** | Operativo | Carga artefactos y esquemas predefinidos desde disco/memoria. | Limitado exclusivamente a los fixtures registrados para pruebas. |
| **`synthesizer`** | Operativo | Sintetizador heurístico determinista basado en reglas estructurales (AST a Semantic IR v0.5). | **No es IA**: genera transformaciones mecánicas elementales (e.g., `param * 2`, inicialización de records y defaults primitivos). No interpreta lenguaje natural libre. |
| **`real`** | Pendiente (Stub) | Conector HTTP para proveedor LLM externo (`HttpLlmProvider`). | Falla explícitamente (`AgentError::Http`) indicando que el transporte de red no está configurado. No declarar operativo hasta implementar `INIT-05-04`. |

---

## 4. Ejemplo Reproducible Paso a Paso

A continuación se muestra un flujo completo end-to-end usando una única especificación Markdown.

### 4.1. Crear la especificación (`spec.md`)

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

### 4.2. Crear la suite de pruebas (`suite.yaml`)

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

### 4.3. Ejecución de los cinco comandos

```bash
# 1. Verificar la especificación
agent-native verify spec.md

# 2. Compilar a componente WebAssembly
agent-native build spec.md -o profile-manager.wasm --mode synthesizer

# 3. Ejecutar función en el runtime independiente
agent-native run -c profile-manager.wasm -f calculate-score 25
# Salida esperada: Function 'calculate-score' returned 1 value(s): [0]: S32(50)

# 4. Validar contra la suite de aceptación
agent-native test -s suite.yaml -c profile-manager.wasm

# 5. Publicar en el almacén de distribución local
agent-native publish -s spec.md -d ./dist --source-wi AN-028 --mode synthesizer
```

Para obtener salida JSON estructurada en cualquiera de estos comandos, anteponer `--format json`:
```bash
agent-native --format json run -c profile-manager.wasm -f calculate-score 25
```
