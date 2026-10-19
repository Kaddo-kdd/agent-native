---
type: adr
id: ADR-006
status: accepted
date: 2026-10-19
related_work_items:
  - AN-031
code:
  - .github/workflows/
  - crates/cli/
---

# ADR-006 — Empaquetado, firma y verificación para distribución multiplataforma

## Context

Con la culminación de INIT-05-01 (CLI funcional en AN-028) e INIT-05-02 (API pública y ejemplo de embebido en AN-029 y AN-030), el proyecto `agent-native` alcanza la etapa de distribución autónoma ([`INI-005`](../initiatives/INI-005-rm-05-standalone-cli-runtime-api.md)).

Para materializar la promesa de "ejecución sin toolchain ni dependencias externas" documentada en `business.md`, es indispensable proveer binarios precompilados para las plataformas de desarrollo estándar. Sin embargo, distribuir binarios ejecutables plantea desafíos críticos:

1. **Integridad y No-Repudio:** El usuario o script de instalación debe poder verificar matemáticamente que el binario descargado no ha sido alterado, truncado ni comprometido por intermediarios en la red.
2. **Entornos Offline y Air-Gapped:** Fiel a las invariantes de ADR-001 y ADR-002, el protocolo de verificación de integridad debe poder ejecutarse en máquinas aisladas sin requerir conexión a proveedores de identidad OIDC o servidores de claves centralizados.
3. **Pregunta Abierta OQ-05-01:** Selección de la tecnología de firma criptográfica (`minisign` vs `cosign`/Sigstore vs `gpg`).
4. **Restricción de Identidad (Decisión D3, 2026-10-19):** El nombre comercial está en revisión humana; los artefactos se estructuran bajo el nombre interno `agent-native` sin publicar releases públicos definitivos ni subir paquetes a crates.io hasta recibir autorización formal de branding.

## Options Considered

### 1. Mecanismo de Firma Criptográfica (Resolución OQ-05-01)

#### Opción A — GnuPG / PGP
- **Pros:** Estándar histórico ampliamente reconocido en distribuciones Linux tradicionales.
- **Contras:** Toolchain pesada y compleja de automatizar de forma determinista en CI; servidores de claves (keyservers) poco fiables y propensos a desincronización; superficie de configuración propensa a algoritmos legados débiles; mala experiencia de usuario en Windows y macOS.
- **Veredicto:** **Rechazada**.

#### Opción B — Cosign / Sigstore (Keyless OIDC)
- **Pros:** Estándar moderno de la OpenSSF, ideal para ecosistemas OCI/contenedores; flujo *keyless* vinculado al token OIDC de GitHub Actions.
- **Contras:** Requiere conexión activa en tiempo de verificación contra autoridades de certificación (Fulcio) y registros de transparencia (Rekor); no funciona de manera nativa en entornos completamente offline o air-gapped sin configurar mirrors locales complejos; el binario de verificación `cosign` es pesado (>100 MB); complejidad excesiva para artefactos CLI standalone.
- **Veredicto:** **Rechazada para la distribución del binario CLI standalone**.

#### Opción C — Minisign (Ed25519)
- **Pros:**
  - Criptografía moderna y robusta: firmas Ed25519 con verificación determinista.
  - 100% offline: la verificación depende únicamente de la clave pública distribuida del proyecto (`minisign.pub`), sin dependencias de red, OIDC ni servidores de certificados.
  - Extremadamente ligero: herramienta CLI de ~50 KB y librerías nativas en Rust (`minisign-verify`) sin dependencias complejas.
  - Adoptado por toolchains modernas y proyectos de sistemas (Zig, runtimes de WebAssembly, OpenBSD).
  - Admite comentarios confiables (*trusted comments*) que garantizan la procedencia del artefacto.
- **Contras:** Requiere gestión segura de la clave privada de release mediante secretos de repositorio (`RELEASE_MINISIGN_SECRET_KEY`) y contraseña asociada.
- **Veredicto:** **Seleccionada**.

---

### 2. Triples Objetivo y Formatos de Compresión

Se consideran los 3 triples canónicos definidos en el alcance de RM-05:

| Triple Objetivo | Sistema Operativo / Arquitectura | Formato de Empaquetado | Extensión |
| :--- | :--- | :--- | :--- |
| `x86_64-unknown-linux-gnu` | Linux x86_64 (glibc 2.31+) | Tarball gzip | `.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows x86_64 | Archivo Zip | `.zip` |
| `aarch64-apple-darwin` | macOS Apple Silicon (M1/M2/M3/M4) | Tarball gzip | `.tar.gz` |

*Nota:* Variantes adicionales como `x86_64-unknown-linux-musl` y `x86_64-apple-darwin` (Intel) quedan categorizadas como secundarias y diferidas a requerimientos futuros sin alterar la arquitectura base.

---

### 3. Esquema de Manifiesto y Firma Unificada

- **Opción A (Firma individual por binario):** Firmar cada `.tar.gz` o `.zip` con un archivo `.minisig` individual (e.g. `agent-native-x86_64-linux.tar.gz.minisig`).
  - *Problema:* Multiplica los artefactos descargables y complica la automatización de verificación en scripts.
- **Opción B (Manifiesto unificado `SHA256SUMS` firmado):** Generar un único archivo `SHA256SUMS` con los hashes SHA-256 de todos los archivos empaquetados del release, y firmar exclusivamente dicho manifiesto generando `SHA256SUMS.minisig`.
  - *Ventaja:* El usuario o script descarga el manifiesto y su firma una sola vez; verifica la firma del manifiesto mediante Minisign y luego verifica el hash del archivo descargado con utilidades nativas (`sha256sum`, `shasum` o PowerShell `Get-FileHash`).
  - **Veredicto:** **Seleccionada Opción B**.

---

## Decision

Se aprueba formalmente la siguiente arquitectura de empaquetado, firma y verificación:

### 1. Convención de Nomenclatura de Artefactos
Para una versión `<VERSION>` (e.g. `v0.1.0`), los archivos de release deben seguir la convención:
- Linux: `agent-native-<VERSION>-x86_64-unknown-linux-gnu.tar.gz`
- macOS: `agent-native-<VERSION>-aarch64-apple-darwin.tar.gz`
- Windows: `agent-native-<VERSION>-x86_64-pc-windows-msvc.zip`

### 2. Contenido Canónico del Paquete
Cada archivo comprimido contendrá estrictamente en su raíz:
- `agent-native` (o `agent-native.exe` en Windows): binario compilado en modo `release`, con optimización LTO, símbolos depurados (`strip = true`) y sin dependencias dinámicas fuera de las del sistema base.
- `LICENSE`: texto de la licencia del proyecto (MIT).
- `README.md`: guía rápida de uso y referencia a los comandos principales (`build`, `verify`, `test`, `run`, `publish`).

### 3. Protocolo de Integridad y Firma (Minisign)
1. Durante el job de finalización del release en CI (AN-032), se computan los hashes SHA-256 de todos los archivos comprimidos y se genera el archivo:
   ```text
   SHA256SUMS
   ```
2. Se firma el archivo `SHA256SUMS` utilizando `minisign` con la clave secreta del proyecto inyectada vía GitHub Secrets:
   ```bash
   minisign -S -s release.key -m SHA256SUMS -t "agent-native release <VERSION>"
   ```
   produciendo el archivo de firma:
   ```text
   SHA256SUMS.minisig
   ```
3. La clave pública de verificación se publica permanentemente en el repositorio en `knowledge/tech/release-signing-key.pub` y en la documentación del producto.

### 4. Protocolo de Verificación del Consumidor
Un usuario o script de instalación verifica el artefacto descargado en dos pasos deterministas:

```bash
# 1. Verificar autenticidad e integridad del catálogo de sumas
minisign -Vm SHA256SUMS -p release-signing-key.pub

# 2. Verificar el binario descargado contra el hash firmado
sha256sum --check --ignore-missing SHA256SUMS
```

En entornos Windows PowerShell:
```powershell
# 1. Verificar firma
minisign.exe -Vm SHA256SUMS -p release-signing-key.pub

# 2. Verificar hash local
$expected = (Get-Content SHA256SUMS | Select-String "agent-native-.*-windows.*\.zip").ToString().Split(" ")[0]
$actual = (Get-FileHash -Algorithm SHA256 agent-native-*-windows.zip).Hash.ToLower()
if ($expected -ne $actual) { throw "Integrity check failed" }
```

### 5. Respeto a la Restricción de Identidad (Decisión D3)
- Toda la configuración de release y empaquetado utiliza el nombre técnico `agent-native`.
- El pipeline automatizado (AN-032) queda preparado para operar sobre tags y builds de validación interna.
- Se mantiene congelada la publicación en `crates.io`, la distribución masiva de instaladores automáticos (`brew`, `winget`, `scoop`) y el anuncio público hasta que se tome la decisión humana de naming definitivo.

---

## Consequences

### Positive
- **Independencia Total de Red:** La verificación criptográfica es 100% determinista y operable en entornos desconectados o restringidos.
- **Auditoría Centralizada:** Un único archivo `SHA256SUMS.minisig` asegura todos los paquetes de un release sin duplicar firmas.
- **Ligereza y Ergonomía:** Minisign no requiere configuración de demonios, infraestructuras complejas ni contenedores pesados.
- **Preparación Limpia para AN-032:** El pipeline de GitHub Actions tiene especificaciones exactas para su matriz de compilación y steps de firma.

### Negative / Trade-offs
- Requiere custodia y rotación cuidadosa de la clave secreta Ed25519 en los secretos de la organización.
- Los usuarios sin `minisign` instalado deben descargar la utilidad ligera o utilizar herramientas compatibles con Ed25519 (o confiar únicamente en el hash SHA-256 si operan en modo degradado).

---

## Validation Plan

1. Verificar que `minisign` genere y valide firmas reproducibles en los tres entornos objetivo.
2. Comprobar que el archivo `SHA256SUMS` sea generado de manera compatible con utilidades estándar GNU `sha256sum`, macOS `shasum -a 256` y PowerShell `Get-FileHash`.
3. Validar la integración del proceso de empaquetado y firma en el pipeline de CI/CD que se implementará en **AN-032**.
