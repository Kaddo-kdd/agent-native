# Guía de Verificación de Integridad y Firma de Releases

Esta guía describe el procedimiento determinista y 100% offline para verificar la autenticidad e integridad de los binarios distribuidos de `agent-native`, conforme a las directivas de [ADR-006](decisions/ADR-006-empaquetado-firma-verificacion.md).

---

## 1. Triples y Artefactos de Distribución

En cada release se generan y publican los siguientes archivos:

| Triple / Archivo | Plataforma | Contenido |
| :--- | :--- | :--- |
| `agent-native-<VERSION>-x86_64-unknown-linux-gnu.tar.gz` | Linux x86_64 | Binario `agent-native`, `LICENSE`, `README.md` |
| `agent-native-<VERSION>-x86_64-pc-windows-msvc.zip` | Windows x86_64 | Binario `agent-native.exe`, `LICENSE`, `README.md` |
| `agent-native-<VERSION>-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon | Binario `agent-native`, `LICENSE`, `README.md` |
| `SHA256SUMS` | Todas | Manifiesto de sumas de comprobación SHA-256 de los 3 paquetes |
| `SHA256SUMS.minisig` | Todas | Firma criptográfica Ed25519 del archivo `SHA256SUMS` |

---

## 2. Clave Pública de Firma

La clave pública oficial del proyecto se encuentra disponible en el repositorio en:
[`knowledge/tech/release-signing-key.pub`](release-signing-key.pub)

```text
untrusted comment: agent-native release public key
RWS0c/hY6W0qXo5Y7Z1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t
```

---

## 3. Procedimiento de Verificación en Linux / macOS

### Requisitos previos:
- Utilidad `minisign` instalada (`apt install minisign`, `brew install minisign`, o binario estático).
- Utilidad estándar `sha256sum` (Linux) o `shasum` (macOS).

### Paso 1: Descargar los artefactos
Descargue el paquete de su plataforma, el archivo `SHA256SUMS` y la firma `SHA256SUMS.minisig`.

### Paso 2: Verificar la firma del catálogo SHA256SUMS
```bash
minisign -Vm SHA256SUMS -p release-signing-key.pub
```
*Salida esperada:*
```text
Signature and comment signature verified
Trusted comment: agent-native release <VERSION>
```

### Paso 3: Verificar la integridad del paquete descargado
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

## 4. Procedimiento de Verificación en Windows (PowerShell)

### Paso 1: Verificar la firma con `minisign.exe`
```powershell
.\minisign.exe -Vm SHA256SUMS -p release-signing-key.pub
```

### Paso 2: Verificar el hash SHA-256 del paquete zip
```powershell
$expectedHash = (Get-Content SHA256SUMS | Select-String "agent-native-.*-windows.*\.zip").ToString().Split(" ")[0].Trim()
$actualHash = (Get-FileHash -Algorithm SHA256 .\agent-native-*-windows-msvc.zip).Hash.ToLower()

if ($expectedHash -eq $actualHash) {
    Write-Host "✓ Integridad verificada exitosamente: el hash coincide con el catálogo firmado." -ForegroundColor Green
} else {
    Write-Error "Fallo de integridad: el hash local ($actualHash) no coincide con el catálogo firmado ($expectedHash)."
}
```

---

## 5. Garantías de Seguridad y Operación Offline

- **Sin Conexión en Verificación:** El proceso de verificación no efectúa ninguna petición de red. No depende de servidores de claves externos ni proveedores de identidad OIDC.
- **Fail-Explicit:** Cualquier discrepancia en el contenido, truncamiento en la descarga o manipulación de bytes provoca el fallo inmediato de `sha256sum` o `minisign`.
- **Procedencia Inmutable:** Los comentarios confiables (*trusted comments*) de Minisign asocian la firma con el tag de release exacto generado en el pipeline automatizado.
