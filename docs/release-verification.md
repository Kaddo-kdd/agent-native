# Verificación de Integridad y Firmas de Release en Wasma

Este documento describe el modelo de integridad criptográfica, los procedimientos de verificación para consumidores, la diferenciación de firmas, la política de custodia de claves y los controles de aprobación humana de Wasma.

---

## 1. Distinción Conceptual Fundamental

Es crucial distinguir dos conceptos independientes dentro del ecosistema de Wasma:

| Concepto | Ámbito | Herramienta | Propósito |
| :--- | :--- | :--- | :--- |
| **Verificación de Especificación IR** | Lenguaje y Componente | `wasma verify <spec.json>` | Valida sintáctica y semánticamente el Semantic IR contra esquemas normativos (v0.4, v0.5, v0.6-exp) y comprueba el determinismo del compilador WebAssembly. |
| **Verificación Criptográfica de Release** | Distribución de Artefactos | `minisign` + `sha256sum` | Comprueba que los binarios compilados y empaquetados no han sido manipulados, provienen del pipeline de construcción legítimo y coinciden bit a bit con los hashes firmados. |

> [!IMPORTANT]
> `wasma verify` no verifica firmas de artefactos de release ni sustituye a `minisign`. Del mismo modo, la verificación criptográfica con `minisign` no analiza la corrección semántica del código WebAssembly contenido en los binarios.

---

## 2. Procedimiento de Verificación para Consumidores

Para validar un paquete de release descargado, el consumidor debe verificar primero la autenticidad del catálogo `SHA256SUMS` mediante `minisign`, y luego comprobar el hash del archivo descargado con `sha256sum`.

### 2.1 Requisitos Previos

Instalar `minisign` y las utilidades estándar de checksum:

- **Linux (Debian/Ubuntu):**
  ```bash
  sudo apt-get install minisign coreutils
  ```
- **macOS (Homebrew):**
  ```bash
  brew install minisign
  ```
- **Windows (Scoop / Winget / Chocolatey):**
  ```powershell
  scoop install minisign
  # o
  winget install jedisct1.minisign
  ```

### 2.2 Archivos Necesarios

Descargar desde la release de GitHub (`https://github.com/Kaddo-kdd/wasma/releases`):
1. El archivo binario de la plataforma deseada:
   - Linux x86_64: `wasma-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
   - Windows x86_64: `wasma-v0.1.0-x86_64-pc-windows-msvc.zip`
   - macOS arm64: `wasma-v0.1.0-aarch64-apple-darwin.tar.gz`
2. El catálogo de checksums: `SHA256SUMS`
3. La firma Minisign: `SHA256SUMS.minisig`
4. La clave pública oficial de release (disponible en el repositorio en `knowledge/tech/release-signing-key.pub`).

### 2.3 Paso 1: Verificar la Firma Criptográfica del Catálogo

Ejecutar `minisign` indicando el catálogo (`-m`) y la clave pública oficial (`-p` o `-P`):

```bash
minisign -Vm SHA256SUMS -p release-signing-key.pub
```

O utilizando directamente la cadena de clave pública:
```bash
minisign -Vm SHA256SUMS -P "<CLAVE_PUBLICA_OFICIAL>"
```

**Salida esperada:**
```text
Signature and comment signature verified
Trusted comment: wasma release v0.1.0
```

Si el catálogo fue alterado o la firma no corresponde a la clave oficial, `minisign` fallará con un error explícito y código de retorno distinto de cero:
```text
Signature verification failed
```

### 2.4 Paso 2: Verificar la Integridad del Artefacto de Release

Una vez comprobado que `SHA256SUMS` es auténtico e inalterado, validar los binarios empaquetados:

**En Linux / macOS:**
```bash
sha256sum -c SHA256SUMS --ignore-missing
```

**En Windows (PowerShell):**
```powershell
Get-FileHash wasma-v0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
# Comparar el hash obtenido con la entrada correspondiente en SHA256SUMS
```

**Salida esperada:**
```text
wasma-v0.1.0-x86_64-unknown-linux-gnu.tar.gz: OK
```

---

## 3. Modos de Firma y Aislamiento de Secretos en CI

El workflow `.github/workflows/release.yml` implementa una separación estricta por diseño entre validación rutinaria y firma productiva oficial:

### 3.1 Control Estricto de Inventario de Paquetes
Previo a la generación del catálogo de checksums y la firma, el job `validate-and-sign` ejecuta un control exhaustivo del inventario de paquetes en `dist/`:
- **Determinación canónica de `TAG`:** Derivado de `GITHUB_REF` y validado frente a la versión de `wasma-cli` en Cargo (`0.1.0`).
- **Comprobación de paquetes esperados:** Exige exactamente los tres archivos canónicos:
  * `wasma-${TAG}-x86_64-unknown-linux-gnu.tar.gz`
  * `wasma-${TAG}-x86_64-pc-windows-msvc.zip`
  * `wasma-${TAG}-aarch64-apple-darwin.tar.gz`
- **Rechazo determinista:** Cualquier archivo faltante, paquete adicional, artefacto duplicado o archivo con versión diferente provoca el fallo inmediato del pipeline con salida de error explícita.

### 3.2 Modo de Validación (`mode: validation` — Dry-Run)
- **Propósito:** Validar el empaquetado, la generación de `SHA256SUMS` y el flujo criptográfico end-to-end en entornos de CI sin requerir ni exponer secretos productivos.
- **Aislamiento de secretos:** Este paso **no tiene acceso a secretos oficiales** en su bloque `env`. Los secretos no se inyectan en el entorno del proceso.
- **Mecanismo:** Genera un par de claves Minisign efímero (`minisign -G`) en memoria temporal.
- **Firma producida:** `SHA256SUMS.test-minisig` (con extensión `.test-minisig` explícita).
- **Comentario confiable:** `"wasma TEST DRY-RUN signature (NOT FOR OFFICIAL DISTRIBUTION)"`.
- **Artefactos exportados:** Incluye `test-minisign.pub` en el bundle de validación para trazabilidad.

### 3.3 Modo Oficial (`mode: official`)
- **Propósito:** Generar la firma criptográfica canónica de distribución para releases públicas.
- **Acceso a secretos:** Exclusivamente el paso `Execute Minisign Official Signing (Guarded Mode)` recibe los secretos canónicos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD`.
- **Firma producida:** `SHA256SUMS.minisig`.
- **Controles de seguridad obligatorios (Gating):**
  1. `confirm_publish == 'true'`: Intención de ejecución del operador formalmente declarada.
  2. `GITHUB_REF` válido: Debe ejecutarse sobre un tag de release que cumpla `refs/tags/v*`.
  3. Correspondencia de versión: El tag debe coincidir exactamente con la versión de `wasma-cli` en Cargo (`0.1.0`).
  4. Clave pública auténtica: El archivo `knowledge/tech/release-signing-key.pub` no debe contener el marcador `PLACEHOLDER`.
  5. Secreto configurado: El secreto `RELEASE_MINISIGN_SECRET_KEY` debe estar presente en el entorno de GitHub Actions.
  6. Memoria segura: La clave privada se procesa en un directorio temporal aislado (`chmod 700 / 600`), con `set +x` para evitar fugas en logs y destrucción segura mediante `trap ... shred -u`.

---

## 4. Política de Custodia, Rotación y Revocación de Claves

### 4.1 Custodia de Claves
- La clave privada de release **nunca** debe almacenarse en repositorios de código ni compartirse en canales inseguros.
- El almacenamiento maestro de la clave privada debe residir en hardware seguro offline (HSM o dispositivo de almacenamiento cifrado bajo custodia del mantenedor principal).
- La copia utilizada en CI reside exclusivamente como secreto cifrado en GitHub Actions (`RELEASE_MINISIGN_SECRET_KEY`), protegido con contraseña (`RELEASE_MINISIGN_PASSWORD`).

### 4.2 Provisión Inicial de Claves (Requisito Humano Pendiente)
Actualmente, el archivo `knowledge/tech/release-signing-key.pub` contiene un marcador de posición (`PLACEHOLDER`). Para activar la primera release oficial:
1. El mantenedor autorizado debe ejecutar localmente:
   ```bash
   minisign -G -p release-signing-key.pub -s release-signing-key.key
   ```
2. Configurar en el repositorio de GitHub los secretos:
   - `RELEASE_MINISIGN_SECRET_KEY`: Contenido del archivo `release-signing-key.key`.
   - `RELEASE_MINISIGN_PASSWORD`: Contraseña asignada al generar la clave.
3. Reemplazar `knowledge/tech/release-signing-key.pub` en el repositorio con la clave pública generada mediante un commit auditado y revisado.

### 4.3 Procedimiento de Rotación de Claves
- Si se programa una rotación de rutina:
  1. Generar un nuevo par de claves Minisign.
  2. Publicar la nueva clave pública en `knowledge/tech/release-signing-key.pub`.
  3. Registrar la transición y huella digital en el `CHANGELOG.md` de la versión correspondiente.
  4. Actualizar los secretos en GitHub Actions.

### 4.4 Procedimiento ante Sospecha de Compromiso (Revocación)
- En caso de sospecha razonable de compromiso de la clave privada:
  1. Eliminar inmediatamente los secretos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD` de la configuración del repositorio en GitHub.
  2. Publicar una notificación de seguridad (Security Advisory) detallando las versiones afectadas.
  3. Deshabilitar los flujos de release automáticos.
  4. Generar un nuevo par de claves y desplegar una versión de parche correctiva con la nueva clave pública.

---

## 5. Arquitectura del Gate de Aprobación Humana y Salvaguarda D3

### 5.1 Distinción entre `confirm_publish` y GitHub Environment
Es fundamental entender la arquitectura en dos niveles para la autorización de releases:

```
[Trigger manual (workflow_dispatch)]
       │
       ▼
  confirm_publish: true  ──► (Nivel 1: Intención del operador en CI)
       │
       ▼
 [validate-and-sign (official)]
       │
       ▼
[Job publish: Environment 'production'] ──► (Nivel 2: Aprobación formal independiente)
       │                                       Requiere:
       │                                       - Aprobación explícita de Required Reviewers
       │                                       - Acceso restringido a secretos de release
       ▼
[GitHub Releases Publicación]
```

> [!WARNING]
> El parámetro de entrada `confirm_publish: true` representa únicamente la **intención de ejecución** del operador al disparar el workflow. **No sustituye ni anula** la aprobación independiente que otorga un GitHub Environment protegido con revisores humanos requeridos (*Required Reviewers*).

### 5.2 Salvaguarda Activa (Decisión D3)
Conforme a la **Decisión D3** de gobernanza del proyecto:
- El job `publish` en `.github/workflows/release.yml` permanece formalmente congelado con `if: false`.
- Ningún pipeline automatizado tiene autorización de publicar releases en GitHub ni en registros públicos sin aprobación humana previa y explícita.
- La eliminación de esta salvaguarda requerirá una decisión formal del equipo de desarrollo, la verificación completa de los criterios de aceptación de la iniciativa INI-009, la configuración del Environment protegido con revisores humanos y la provisión exitosa de las claves de firma oficiales (previstas para AN-076).
