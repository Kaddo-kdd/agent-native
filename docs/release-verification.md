# Verificación de Integridad y Firmas de Release en Wasma

Este documento describe el modelo de integridad criptográfica, los procedimientos de verificación para consumidores, la arquitectura de jobs en GitHub Actions, la política de custodia humana de claves y los controles de aprobación escalonada de Wasma.

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
3. La firma Minisign oficial: `SHA256SUMS.minisig`
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

## 3. Arquitectura del Pipeline y Segregación de Responsabilidades

El workflow [`.github/workflows/release.yml`](file:///.github/workflows/release.yml) está modularizado en 4 jobs con estricto principio de menor privilegio:

```
[build (Linux, Windows, macOS)] ──► package-* (sin secretos)
              │
              ▼
[validate] ───────────────────────► release-bundle (sin secretos, firma efímera en validation)
              │
              ▼ (si mode == official)
[sign-official] ──────────────────► Environment 'release' (Required Reviewers, actualmente if: false)
              │                     Produce: official-release-bundle (SHA256SUMS.minisig)
              ▼
[publish] ────────────────────────► GitHub Release formal (Decisión D3: if: false hasta AN-076)
```

### 3.1 Job `build`: Compilación y Empaquetado
- Compilación determinista bloqueada (`cargo build --release --locked`).
- Validación de arquitectura real (`arm64` en macOS).
- Normalización de permisos (`755` ejecutable, `644` docs) y timestamps fijos.
- Empaquetado con GNU tar (`--sort=name`) en Unix y PowerShell `Compress-Archive` en Windows.
- Eliminación estricta del directorio de staging temporal inmediatamente tras la verificación de cada archivo comprimido.
- Subida de artefactos específicos (`path: wasma-*.${{ matrix.archive_ext }}`).
- **Cero acceso a secretos.**

### 3.2 Job `validate`: Integridad de Inventario y Checksums
- Descarga los 3 paquetes de plataforma.
- Inspecciona la totalidad de entradas en `dist/` (`ls -1A | LC_ALL=C sort`): rechaza directorios, enlaces simbólicos, archivos extra o paquetes faltantes.
- Genera el catálogo determinista `SHA256SUMS` (exactamente 3 entradas).
- Comprueba los hashes con `sha256sum -c SHA256SUMS`.
- **Modo Validation (Dry-Run):** Genera claves efímeras en memoria (`minisign -G`), firma `SHA256SUMS.test-minisig` y valida la firma.
- Exporta el artefacto `release-bundle` (bundle validado sin firma oficial).
- **Cero acceso a secretos.**

### 3.3 Job `sign-official`: Firma Oficial Criptográfica (Gate Humano)
- **Estructura:** Asociado al GitHub Environment `release`.
- **Protección activa:** Deshabilitado mediante `if: false` hasta la autorización humana formal y configuración de Required Reviewers.
- Requisitos previos para su habilitación:
  1. Environment `release` existente en GitHub.
  2. *Required Reviewers* configurados con los mantenedores autorizados requeridos.
  3. *Prevent self-review* habilitado cuando exista un segundo mantenedor.
  4. *Bypass administrativo* deshabilitado.
  5. Restricción del Environment exclusivamente a tags `refs/tags/v*`.
  6. Autorización humana independiente mediante aprobación en GitHub Actions.
  7. Secretos oficiales `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD` aprovisionados en el Environment.
- Operación criptográfica: Re-verifica inventario y checksums, comprueba correspondencia de versión Cargo y tag, verifica que `release-signing-key.pub` no tenga `PLACEHOLDER`, opera sobre sistema de archivos en RAM (`tmpfs`), genera `SHA256SUMS.minisig` oficial y produce `official-release-bundle`.

### 3.4 Job `publish`: Publicación Formal en GitHub Releases
- Reservado para **AN-076**.
- Mantenido estrictamente con `if: false` bajo la **Decisión D3**.
- Requerirá un gate de publicación independiente del gate de firma, asegurando que la firma criptográfica y la liberación al público constituyan decisiones humanas separadas.

---

## 4. Trazabilidad de Versión y Nomenclatura de Artefactos

Para garantizar que ningún artefacto intermedio oculte su procedencia, se implementa una nomenclatura determinista:

$$\text{TAG} = \begin{cases}
\text{Git Tag (e.g. } \texttt{v0.1.0}\text{)}, & \text{si el evento es } \texttt{refs/tags/v*} \\
\texttt{v}\{\text{Cargo Version}\}\texttt{-dev-}\{\text{Short SHA}\}, & \text{en ramas, PRs o pruebas manuales}
\end{cases}$$

### Reglas de Trazabilidad:
1. **Releases Oficiales:** Exigen correspondencia biunívoca exacta entre el tag de Git (`v0.1.0`) y la versión declarada en `Cargo.toml` (`0.1.0`). Cualquier discrepancia aborta el pipeline inmediatamente.
2. **Pruebas y Validación:** Utilizan el esquema `v{CARGO_VERSION}-dev-{SHORT_SHA}` (ejemplo: `v0.1.0-dev-d3e0d54`).
3. **Metadatos de Procedencia:** Los logs de CI y los bundles registran explícitamente tanto el `SHORT_SHA` (7 caracteres para legibilidad) como el `FULL_COMMIT_SHA` (40 caracteres hexadecimales para trazabilidad criptográfica unívoca).

---

## 5. Protocolo de Custodia Humana de Claves Minisign

> [!CAUTION]
> **Aislamiento Absoluto de Claves Maestras:** La clave privada Minisign y su contraseña son de custodia humana exclusiva. Nunca deben entregarse, transferirse, pegarse ni mostrarse al asistente de IA, ni almacenarse en repositorios Git ni logs de ejecución.

### 5.1 Advertencia sobre Medios de Almacenamiento y SSDs
No se deben almacenar claves maestras en directorios temporales no cifrados del sistema operativo ni confiar exclusivamente en utilidades como `shred` o `srm` para su destrucción en unidades de estado sólido (SSD / NVMe / eMMC). Debido al algoritmo de nivelación de desgaste (*wear-leveling*) y la capa de traducción de flash (*Flash Translation Layer* / FTL), los bloques sobreescritos se reasignan lógicamente en lugar de ser eliminados físicamente de las celdas NAND, dejando copias residuales en memoria no volátil.

### 5.2 Procedimiento de Generación Local (Entorno Aislado)
El mantenedor humano debe ejecutar en una estación de trabajo confiable, preferiblemente offline o aislada:

```bash
# 1. Crear directorio en memoria RAM (tmpfs) para que la clave nunca toque el disco físico:
mkdir -p -m 700 /tmp/wasma-key-ram
sudo mount -t tmpfs -o size=10M,mode=0700 tmpfs /tmp/wasma-key-ram
cd /tmp/wasma-key-ram

# 2. Generar el par de claves Minisign:
#    Se solicitará una contraseña robusta (mínimo 24 caracteres aleatorios / Diceware)
minisign -G -p wasma-release.pub -s wasma-release.key

# 3. Exportar inmediatamente la clave privada al medio de respaldo seguro (ver 5.3)
```

### 5.3 Respaldo y Almacenamiento Seguro (Cold Storage)
1. **Almacenamiento Primario:** Gestor de contraseñas corporativo/personal con cifrado de conocimiento cero (*zero-knowledge*), autenticación multifactor (MFA por token físico FIDO2/WebAuthn) y acceso de emergencia formalizado.
2. **Respaldo Frío (Disaster Recovery):** Dispositivo USB físico con cifrado por hardware (o partición LUKS / BitLocker) depositado en caja fuerte física bajo control del mantenedor responsable.
3. **Limpieza del Entorno Temporal:**
   ```bash
   shred -u wasma-release.key 2>/dev/null || rm -f wasma-release.key
   cd ~
   sudo umount /tmp/wasma-key-ram
   rm -rf /tmp/wasma-key-ram
   ```

### 5.4 Procedimiento de Provisión en GitHub Actions
La clave pública y los secretos se provisionan de manera humana e independiente:
1. **Clave Pública:** El mantenedor copia el contenido de `wasma-release.pub` (archivo de texto público de 2 líneas) y sustituye el `PLACEHOLDER` en [`knowledge/tech/release-signing-key.pub`](file:///knowledge/tech/release-signing-key.pub) mediante un commit firmado y auditado.
2. **Environment en GitHub:**
   - Navegar a **Settings → Environments → New environment**.
   - Nombre: `release`.
   - Activar **Required reviewers** (asignar mantenedores autorizados requeridos).
   - Activar **Prevent self-review** (cuando exista un segundo mantenedor).
   - Desactivar **Allow administrators to bypass configured rules**.
   - Configurar **Deployment branches**: Restringir a tags `refs/tags/v*`.
3. **Secretos del Environment (NO secrets de repositorio):**
   - `RELEASE_MINISIGN_SECRET_KEY`: Contenido completo de `wasma-release.key`.
   - `RELEASE_MINISIGN_PASSWORD`: Contraseña asignada a la clave privada.

---

## 6. Procedimiento de Rotación y Revocación

### 6.1 Rotación Programada
1. Generar un nuevo par de claves bajo el protocolo 5.2.
2. Actualizar `knowledge/tech/release-signing-key.pub` en el repositorio.
3. Registrar la nueva clave y fecha efectiva en `CHANGELOG.md`.
4. Actualizar los secretos en el GitHub Environment `release`.

### 6.2 Revocación por Sospecha de Compromiso
1. Eliminar inmediatamente los secretos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD` del Environment `release`.
2. Publicar un GitHub Security Advisory y emitir aviso en `wasma.org`.
3. Deshabilitar los pipelines de release en GitHub Actions.
4. Generar un nuevo par de claves y lanzar un parche correctivo con la nueva clave pública.

---

## 7. Salvaguarda Activa de Gobernanza (Decisión D3)

Conforme a la **Decisión D3**:
- Tanto el job `sign-official` como el job `publish` permanecen deshabilitados con `if: false`.
- La activación de `sign-official` requiere la provisión formal de claves y la configuración del Environment `release`.
- La activación de `publish` está reservada para **AN-076**, requiriendo un gate humano independiente y la verificación completa de la iniciativa INI-009.
