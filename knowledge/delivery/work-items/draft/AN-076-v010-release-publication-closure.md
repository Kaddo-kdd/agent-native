---
id: AN-076
type: chore
status: draft
created_at: 2026-10-26T00:00:00.000Z
title: Publicación oficial v0.1.0 y cierre de INI-009
knowledge_level: K4
suggested_knowledge_level: K4
scope_confidence: high
initiative: INI-009
source_roadmap_initiative: INIT-09-09
priority: P0
depends_on:
  - AN-069
  - AN-070
  - AN-071
  - AN-072
  - AN-073
  - AN-075
domains:
  - Distribution & Release
expected_value: >-
  Formalizar la publicación oficial de la versión v0.1.0 en GitHub Releases tras validar
  migración de identidad completa, firma, empaquetado, portal, documentación y piloto externo.
project_state: ai-assisted
template_version: 1
---

# AN-076 — Publicación oficial v0.1.0 y cierre de INI-009

## Contexto y problema

Work Item de coordinación final para la primera entrega pública v0.1.0 de Wasma. Exige que todas las fases precedentes estén completas y verificadas: consolidación de identidad técnica, migración del repositorio, contrato de versión 0.1.0, pipeline de firma, paquetes multiplataforma, documentación de onboarding, aplicaciones de referencia, transparencia en el portal y validación empírica con desarrolladores externos.

## Prerrequisitos obligatorios de seguridad y custodia humana (Transferidos desde AN-069)

Antes de autorizar la ejecución de la publicación oficial de Wasma v0.1.0, deben cumplirse y auditarse rigurosamente los siguientes prerrequisitos obligatorios bajo custodia humana:

1. **Generación humana y custodia segura de claves Minisign**: Generación en frío y en almacenamiento volátil (`tmpfs` en RAM según `docs/release-verification.md`) del par de claves oficial de producción, con respaldo físico offline. La clave privada y su contraseña nunca deben entregarse a agentes, LLMs ni almacenarse en repositorios Git.
2. **Reemplazo auditado de PLACEHOLDER**: Sustitución controlada y auditada del valor provisional en [`knowledge/tech/release-signing-key.pub`](file:///knowledge/tech/release-signing-key.pub) por la clave pública oficial del proyecto mediante commit firmado.
3. **Configuración y verificación del GitHub Environment `release`**: Configuración del entorno en GitHub con:
   - Required Reviewers específicos del mantenedor humano.
   - Activación de «Prevent self-review».
   - Prohibición estricta de bypass administrativo.
4. **Provisión aislada de secretos canónicos**: Inyección manual de los secretos `RELEASE_MINISIGN_SECRET_KEY` y `RELEASE_MINISIGN_PASSWORD` con alcance exclusivo al Environment `release`.
5. **Configuración de GitHub Rulesets para protección de `main` y tags `v*`**:
   - Protección de la rama `main` contra push directo y exigencia de PR con comprobaciones estrictas de CI.
   - Tag Protection Rule para `v*` impidiendo creación no autorizada, actualización o eliminación de etiquetas de release.
6. **Activación autorizada de `sign-official`**: Sustitución de `if: false` en el job `sign-official` exclusivamente bajo autorización humana documentada.
7. **Ejecución y verificación independiente de firma auténtica**: Ejecución controlada en `mode: official` verificando la firma `SHA256SUMS.minisig` contra la clave pública oficial y el catálogo de hashes. El parámetro `confirm_publish: true` es puramente declarativo y NO se considera equivalente a una aprobación independiente.
8. **Autorización separada para habilitar `publish`**: Retiro de la salvaguarda `if: false` del job `publish` (Decisión D3) únicamente tras confirmar que los paquetes firmados y verificados son íntegros.

## Criterios de aceptación

- AC-01: La migración de identidad técnica y del repositorio se encuentran formalmente validadas, comprobando que los contratos Semantic IR, WIT y las garantías de seguridad no sufrieron regresiones.
- AC-02: Los paquetes binarios firmados y el catálogo de hashes SHA-256 están publicados en GitHub Releases bajo el tag oficial `v0.1.0`, con firma Minisign oficial verificada.
- AC-03: Las notas de versión, changelog, limitaciones conocidas y compatibilidad declarada están publicados en el portal y en GitHub Releases bajo la nomenclatura definitiva.
- AC-04: El portal web refleja de forma transparente la sección del autor y los metadatos de build/commit en el footer sin discrepancias.
- AC-05: Los resultados del piloto de adopción con desarrolladores externos están documentados y todos los defectos bloqueantes identificados han sido resueltos.
- AC-06: Se cuenta con la autorización humana explícita y documentada para la liberación pública oficial, habiendo verificado el cumplimiento de todos los prerrequisitos de seguridad y custodia humana.
- AC-07: Se evalúa y formaliza el cierre de INI-009 con `kaddo_complete_initiative`.

## Fuera de alcance

- Certificación para entornos de producción de misión crítica (Wasma v0.1.0 es una release de adopción e inicio).
