# Guía de instalación y ejecución multiplataforma de Wasma

Esta guía describe el procedimiento formal para descargar, verificar, instalar y ejecutar binarios precompilados de **Wasma** en **Linux x86_64**, **macOS ARM64** y **Windows x86_64**, sin necesidad de contar con Rust o herramientas de compilación instaladas.

---

## 1. Principios de distribución e integridad

### 1.1. Binarios autónomos sin toolchain
Los paquetes de distribución de Wasma incluyen el ejecutable nativo (`wasma` o `wasma.exe`), el archivo de licencia (`LICENSE`) y la documentación básica (`README.md`). El binario está completamente compilado en modo `--release --locked` y no requiere dependencias en tiempo de ejecución de Cargo, Rust o Wasmtime externo.

### 1.2. Política de seguridad: Sin instaladores desatendidos
Por principios estrictos de seguridad en la cadena de suministro de software, **Wasma no distribuye ni promueve scripts de instalación desatendida** del tipo `curl | sh` o `powershell -c "iwr ... | iex"`. Toda instalación se realiza mediante:
1. Descarga del paquete comprimido y su catálogo de integridad.
2. Verificación criptográfica local del hash SHA-256.
3. Descompresión controlada en un directorio de usuario o sistema.
4. Incorporación manual y explícita al `PATH`.

### 1.3. Paquetes de desarrollo vs. Releases oficiales
> [!NOTE]
> **Estado actual del proyecto:** Wasma se encuentra en fase de desarrollo previa a su primera entrega pública oficial (AN-076). Actualmente **no existe una release oficial pública publicada ni enlaces de descarga definitivos**. Los únicos artefactos binarios existentes corresponden a paquetes de validación generados por los workflows de GitHub Actions para verificación técnica.

- **Paquetes de CI / Desarrollo (`wasma-v0.1.0-dev-{SHORT_SHA}-{TARGET}`):** Generados en GitHub Actions durante el desarrollo continuo. Su integridad se valida determinísticamente mediante el catálogo `SHA256SUMS`.
- **Releases Oficiales (`wasma-v0.1.0-{TARGET}`):** Serán publicadas formalmente en el repositorio tras la conclusión de la iniciativa INI-009 bajo tags oficiales `v*`. Incluirán el catálogo `SHA256SUMS` y la firma criptográfica oficial Minisign (`SHA256SUMS.minisig`), verificable con la clave pública del proyecto (consulte [`docs/release-verification.md`](release-verification.md)).

---

## 2. Instalación en Linux (x86_64 GNU)

### 2.1. Requisitos
- Arquitectura x86_64 (AMD64 / Intel 64-bit).
- GNU C Library (glibc) 2.31 o superior (estándar en Ubuntu 20.04+, Debian 11+, Fedora 34+, RHEL 9+).
- Utilidades estándar: `tar`, `sha256sum` (o `shasum`).

### 2.2. Descarga y verificación
Descargue el paquete correspondiente a su plataforma y el archivo de sumas de verificación:

```bash
# Nombre del paquete (ejemplo de versión de desarrollo o release v0.1.0)
PACKAGE="wasma-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"

# Descargar paquete y catálogo
# (Sustituya la URL por el enlace de GitHub Release o CI Artifact correspondiente)
# curl -LO https://github.com/Kaddo-kdd/wasma/releases/download/v0.1.0/${PACKAGE}
# curl -LO https://github.com/Kaddo-kdd/wasma/releases/download/v0.1.0/SHA256SUMS

# 1. Verificar la integridad mediante el catálogo SHA-256
sha256sum --check --ignore-missing SHA256SUMS
```

El resultado debe mostrar:
```text
wasma-v0.1.0-x86_64-unknown-linux-gnu.tar.gz: OK
```

### 2.3. Descompresión e instalación
```bash
# 2. Descomprimir el archivo en un directorio temporal
mkdir -p wasma-dist
tar -xzvf "${PACKAGE}" -C wasma-dist

# 3. Comprobar que el binario es ejecutable
ls -l wasma-dist/wasma

# 4. Instalar en el sistema (requiere permisos sudo)
sudo install -m 0755 wasma-dist/wasma /usr/local/bin/wasma

# Alternativa para usuario sin privilegios root:
# mkdir -p ~/.local/bin
# install -m 0755 wasma-dist/wasma ~/.local/bin/wasma
# export PATH="${HOME}/.local/bin:${PATH}"
```

### 2.4. Smoke test de verificación
```bash
wasma --help
wasma --version
```

---

## 3. Instalación en macOS (Apple Silicon ARM64)

### 3.1. Requisitos
- Equipo Mac con procesador Apple Silicon (M1, M2, M3, M4 o variantes).
- macOS 12 (Monterey) o superior.
- Utilidades estándar: `tar`, `shasum`.

### 3.2. Descarga y verificación
```bash
PACKAGE="wasma-v0.1.0-aarch64-apple-darwin.tar.gz"

# 1. Verificar integridad del archivo
shasum -a 256 --check --ignore-missing SHA256SUMS
```

Resultado esperado:
```text
wasma-v0.1.0-aarch64-apple-darwin.tar.gz: OK
```

### 3.3. Descompresión e instalación
```bash
# 2. Descomprimir el archivo
mkdir -p wasma-dist
tar -xzvf "${PACKAGE}" -C wasma-dist

# 3. Validar arquitectura del binario (Apple Silicon ARM64)
file wasma-dist/wasma
# Debe reportar: Mach-O 64-bit executable arm64

# 4. Instalar en el PATH de usuario o sistema
sudo install -m 0755 wasma-dist/wasma /usr/local/bin/wasma

# 5. Atributo de cuarentena de macOS (Gatekeeper):
# ADVERTENCIA DE SEGURIDAD: Nunca retire el atributo de cuarentena sin haber verificado
# previamente la procedencia del archivo y su hash SHA-256 contra el catálogo oficial.
# Únicamente si el hash coincide y el binario proviene del repositorio verificado:
xattr -d com.apple.quarantine /usr/local/bin/wasma 2>/dev/null || true
```

### 3.4. Smoke test de verificación
```bash
wasma --help
wasma --version
```

---

## 4. Instalación en Windows (x86_64 MSVC)

### 4.1. Requisitos y dependencias de ejecución
- Windows 10, Windows 11 o Windows Server 2022+ (arquitectura x86_64 / 64-bit).
- PowerShell 5.1 o PowerShell 7+.
- **Dependencia de runtime:** El binario oficial para Windows está compilado contra el toolchain MSVC (`x86_64-pc-windows-msvc`), por lo que requiere las bibliotecas de tiempo de ejecución de C/C++ de Visual Studio (`vcruntime140.dll` y `msvcp140.dll`).
  - En ediciones de escritorio de Windows 10 y 11, este componente habitualmente ya se encuentra instalado.
  - En imágenes mínimas de Windows Server o contenedores limpios, si el binario no inicia o reporta falta de DLLs de runtime, instale el [paquete oficial Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).

### 4.2. Descarga y verificación
En una ventana de PowerShell:

```powershell
$Package = "wasma-v0.1.0-x86_64-pc-windows-msvc.zip"

# 1. Calcular el hash SHA-256 del paquete descargado
$CalculatedHash = (Get-FileHash -Path $Package -Algorithm SHA256).Hash.ToLower()

# 2. Comparar contra el catálogo SHA256SUMS
$ExpectedLine = Get-Content SHA256SUMS | Where-Object { $_ -match [regex]::Escape($Package) }
$ExpectedHash = ($ExpectedLine -split "\s+")[0].ToLower()

if ($CalculatedHash -eq $ExpectedHash) {
    Write-Host "✓ Integridad verificada exitosamente: $CalculatedHash" -ForegroundColor Green
} else {
    Write-Error "ERROR: Discrepancia en el hash SHA-256. El archivo puede estar corrupto."
}
```

### 4.3. Descompresión e instalación
```powershell
# 3. Descomprimir en una carpeta local
$InstallDir = "$env:USERPROFILE\bin\wasma"
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Expand-Archive -Path $Package -DestinationPath $InstallDir -Force

# 4. Agregar permanentemente al PATH del usuario (si no está presente)
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", [EnvironmentVariableTarget]::User)
    $env:Path += ";$InstallDir"
    Write-Host "✓ Wasma incorporado al PATH del usuario: $InstallDir" -ForegroundColor Green
}
```

### 4.4. Smoke test de verificación
Abra una nueva consola de PowerShell o CMD:

```powershell
wasma --help
wasma --version
```

---

## 5. Diagnóstico de errores frecuentes (Troubleshooting)

### 5.1. `Permission denied` en Linux o macOS
Si al ejecutar `./wasma` recibe un error de permisos:
```bash
chmod +x wasma
```
*Nota:* Los paquetes oficiales de Wasma ya preservan los permisos `0755` para el binario dentro del archivo comprimido. Este error suele producirse únicamente si el comando de descompresión no conservó los atributos del archivo.

### 5.2. Bloqueo de Gatekeeper en macOS («desarrollador no identificado»)
Al descargar archivos comprimidos mediante navegadores en macOS, el sistema operativo asigna automáticamente el atributo extendido `com.apple.quarantine`.

> [!CAUTION]
> **No eluda la seguridad de Gatekeeper indiscriminadamente.**
> Antes de retirar la cuarentena, compruebe estrictamente:
> 1. Que el paquete fue descargado directamente del repositorio canónico (`https://github.com/Kaddo-kdd/wasma`).
> 2. Que el cálculo `shasum -a 256 wasma-*.tar.gz` coincide exactamente con el valor del catálogo `SHA256SUMS`.

Una vez verificada la procedencia e integridad, puede autorizar la ejecución retirando el atributo:
```bash
xattr -d com.apple.quarantine wasma
```
O autorizarla en **Configuración del Sistema → Privacidad y Seguridad → Permitir de todos modos**.

### 5.3. Discrepancia de checksum SHA-256
Si el comando de verificación falla:
- **No ejecute el binario.**
- Compruebe si la descarga se completó totalmente o se interrumpió de forma prematura.
- Vuelva a descargar el archivo y reintente la verificación.

### 5.4. Variables de entorno de inferencia LLM
Wasma utiliza canónicamente el prefijo `WASMA_LLM_*`:
- `WASMA_LLM_PROVIDER`: Proveedor de inferencia (e.g., `openai`, `anthropic`, `local`).
- `WASMA_LLM_ENDPOINT`: URL del servicio LLM.
- `WASMA_LLM_KEY`: Clave de API.

> [!WARNING]
> Las variables con prefijo heredado `AGENT_NATIVE_*` están obsoletas. Wasma emite una advertencia formal y se retirarán en versiones futuras. Migre siempre a `WASMA_LLM_*`.
