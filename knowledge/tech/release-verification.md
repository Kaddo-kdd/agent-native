# Guía de Verificación de Integridad y Firma de Releases

Esta guía describe el marco normativo y los procedimientos técnicos 100% offline para verificar la autenticidad e integridad de los binarios distribuidos de `agent-native`, conforme a las directivas de [ADR-006](decisions/ADR-006-empaquetado-firma-verificacion.md).

---

## 1. Tres Garantías Diferenciadas

Es crucial distinguir tres niveles independientes de garantía técnica:

1. **Integridad de los Artefactos (SHA-256):**
   Garantiza que el archivo comprimido descargado no ha sido truncado, alterado ni corrompido durante el tránsito. Se verifica comparando el hash calculado localmente contra el catálogo `SHA256SUMS`.
2. **Autenticidad y No-Repudio (Minisign Ed25519):**
   Garantiza que el catálogo `SHA256SUMS` fue producido por el equipo legítimo del proyecto y no por un tercero suplantador. Se verifica mediante la firma criptográfica (`.minisig`) contra la clave pública oficial.
3. **Reproducibilidad del Empaquetado:**
   Garantiza que el contenido interno de cada paquete sigue una estructura canónica determinista (`binario`, `LICENSE`, `README.md`) y que el código fuente compilado con las mismas banderas (`opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`) genera un comportamiento binario consistente. *Nota: No se afirman builds idénticos byte a byte entre diferentes compiladores/OS sin un entorno hermético comprobado.*

---

## 2. Triples y Artefactos de Distribución

En cada ciclo de distribución se generan los siguientes artefactos:

| Triple / Archivo | Plataforma | Contenido |
| :--- | :--- | :--- |
| `agent-native-<VERSION>-x86_64-unknown-linux-gnu.tar.gz` | Linux x86_64 | Binario `agent-native`, `LICENSE`, `README.md` |
| `agent-native-<VERSION>-x86_64-pc-windows-msvc.zip` | Windows x86_64 | Binario `agent-native.exe`, `LICENSE`, `README.md` |
| `agent-native-<VERSION>-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon | Binario `agent-native` (ARM64), `LICENSE`, `README.md` |
| `SHA256SUMS` | Todas | Manifiesto ordenado alfabéticamente con las sumas SHA-256 |
| `SHA256SUMS.minisig` (Oficial) o `SHA256SUMS.test-minisig` (Dry-run) | Todas | Firma criptográfica Ed25519 del catálogo `SHA256SUMS` |

---

## 3. Estado de la Clave Pública de Firma

La clave pública oficial del proyecto se documenta en:
[`knowledge/tech/release-signing-key.pub`](release-signing-key.pub)

> [!IMPORTANT]
> **Estado Actual: Pendiente de Aprovisionamiento Humano (Bloqueante para Releases Oficiales).**
> La clave pública actual registrada en el repositorio es un marcador de posición de especificación.
> El modo de distribución oficial en CI requiere que un mantenedor humano genere un par de claves legítimo mediante `minisign -G`, configure el secreto `RELEASE_MINISIGN_SECRET_KEY` en GitHub y actualice `release-signing-key.pub` con la clave pública real.
> Hasta que esto ocurra, el pipeline opera en **Modo de Validación (Dry-Run)**, el cual utiliza pares de claves temporales de prueba y etiqueta inequívocamente los resultados como artefactos de prueba no aptos para distribución comercial.

---

## 4. Modalidades del Pipeline

El pipeline implementado en `.github/workflows/release.yml` distingue dos modalidades estrictas:

### A. Modo de Validación (Dry-Run / CI Testing)
- **Activación:** Ejecución manual (`workflow_dispatch` con `mode: validation`) o ejecución ante tags `v*`.
- **Comportamiento:**
  - Compila y empaqueta los tres binarios para sus respectivas plataformas.
  - Verifica la arquitectura de cada binario (ej. validación ARM64 en macOS).
  - Genera el catálogo `SHA256SUMS` ordenado de forma determinista y comprueba todos los hashes.
  - Genera una clave Minisign efímera en memoria/directorio temporal seguro (`chmod 700`), firma el catálogo como `SHA256SUMS.test-minisig` con comentario de prueba, verifica la firma contra la clave efímera y destruye la clave privada.
  - Sube los paquetes y el manifiesto como artefactos de GitHub Actions para descarga y auditoría.
  - **Restricción de seguridad:** NUNCA crea una GitHub Release pública ni publica en crates.io.

### B. Modo de Distribución Oficial (Reservado para Autorización Futura)
- **Activación:** Exclusivamente manual vía `workflow_dispatch` con `mode: official` y `confirm_publish: true`.
- **Requisitos bloqueantes:**
  - Decisión D3 formalmente levantada por decisión humana.
  - Secretos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD` configurados en GitHub Secrets.
  - Clave pública legítima aprovisionada en `knowledge/tech/release-signing-key.pub`.
- **Comportamiento:**
  - Falla de inmediato si faltan secretos o si la clave pública sigue siendo un placeholder.
  - Firma `SHA256SUMS` produciendo `SHA256SUMS.minisig`.
  - Verifica la firma contra `knowledge/tech/release-signing-key.pub`. Si falla, el pipeline se detiene con error.
  - En la arquitectura futura, la publicación final a GitHub Releases estará protegida por un GitHub Environment con revisores obligatorios.

---

## 5. Procedimiento de Verificación en Linux / macOS

### Requisitos previos:
- Utilidad `minisign` instalada (`apt install minisign`, `brew install minisign`, o binario estático de Minisign).
- Utilidad estándar `sha256sum` (Linux) o `shasum` (macOS).

### Paso 1: Verificar la firma del catálogo SHA256SUMS
Para una release oficial:
```bash
minisign -Vm SHA256SUMS -p release-signing-key.pub
```
*Salida esperada:*
```text
Signature and comment signature verified
Trusted comment: agent-native release <VERSION>
```

Para un paquete de validación / dry-run (usando la clave de prueba generada `test-minisign.pub` adjunta al bundle):
```bash
minisign -Vm SHA256SUMS -p test-minisign.pub -x SHA256SUMS.test-minisig
```

### Paso 2: Verificar la integridad del paquete descargado
```bash
# En Linux:
sha256sum --check --ignore-missing SHA256SUMS

# En macOS:
shasum -a 256 --check --ignore-missing SHA256SUMS
```
*Salida esperada:*
```text
agent-native-<VERSION>-<PLATFORM>.<EXT>: OK
```

---

## 6. Procedimiento de Verificación en Windows (PowerShell)

### Paso 1: Verificar la firma con `minisign.exe`
```powershell
.\minisign.exe -Vm SHA256SUMS -p release-signing-key.pub
```

### Paso 2: Verificar el hash SHA-256 del paquete zip
```powershell
$manifest = Get-Content SHA256SUMS | Where-Object { $_ -match "agent-native-.*-windows-msvc\.zip" }
$expectedHash = $manifest.Split(" ")[0].Trim().ToLower()
$actualHash = (Get-FileHash -Algorithm SHA256 .\agent-native-*-windows-msvc.zip).Hash.ToLower()

if ($expectedHash -eq $actualHash) {
    Write-Host "✓ Integridad verificada exitosamente: el hash coincide con el catálogo firmado." -ForegroundColor Green
} else {
    Write-Error "Fallo de integridad: hash local ($actualHash) != catálogo ($expectedHash)"
}
```

---

## 7. Garantías de Operación Offline

- **Cero Dependencia de Red en Verificación:** Ni `minisign` ni `sha256sum` efectúan consultas a servidores de claves, endpoints de identidad OIDC ni autoridades de certificación externas.
- **Fail-Explicit:** Cualquier alteración de un solo bit en un paquete comprimido o en el archivo `SHA256SUMS` provoca un fallo inmediato con código de salida distinto de cero.
