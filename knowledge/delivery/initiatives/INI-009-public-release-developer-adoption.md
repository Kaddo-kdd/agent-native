---
type: initiative
id: INI-009
title: INI-009 — Public Release & Developer Adoption
status: draft
knowledge_level: K3
priority: high
domains:
  - Developer Experience, Documentation & Branding
  - Distribution & Release
related_capabilities:
  - Artifact Distribution & Verification
  - Developer Onboarding
  - Community & Pilot Validation
created_at: '2026-10-26'
external_links:
  - knowledge/delivery/roadmap.md
candidates:
  - id: INIT-09-01
    title: Auditoría de preparación de release y contrato de versionamiento
    type: chore
    priority: P0
    suggested_knowledge_level: K3
    expected_value: Versionamiento coherente de crates, CLI y tags sin inconsistencias históricas
    materialized_as: AN-068 (candidate)
    dependencies:
      - INI-001...008
  - id: INIT-09-02
    title: Firma oficial, integridad y controles del pipeline de publicación
    type: feature
    priority: P0
    suggested_knowledge_level: K4
    expected_value: Catálogo SHA-256 y firma Minisign con clave oficial en GitHub Actions
    materialized_as: AN-069 (candidate)
    dependencies:
      - AN-068
  - id: INIT-09-03
    title: Empaquetado, instalación y prerelease multiplataforma
    type: feature
    priority: P0
    suggested_knowledge_level: K3
    expected_value: Binarios reproducibles para Linux x86_64 GNU, Windows x86_64 MSVC y macOS ARM64
    materialized_as: AN-070 (candidate)
    dependencies:
      - AN-069
  - id: INIT-09-04
    title: Sección de autor e identidad editorial del proyecto
    type: feature
    priority: P1
    suggested_knowledge_level: K2
    expected_value: Sección editorial «Sobre el autor» en la landing con contexto profesional
    materialized_as: AN-071 (candidate)
    dependencies:
      - Portal actual
  - id: INIT-09-05
    title: Versión, commit y procedencia del despliegue en el footer
    type: feature
    priority: P1
    suggested_knowledge_level: K2
    expected_value: Metadatos generados en build (versión web, SHA corto trazable a commit de GitHub)
    materialized_as: AN-072 (candidate)
    dependencies:
      - Portal actual
  - id: INIT-09-06
    title: Onboarding, quickstart y reconciliación de documentación
    type: docs
    priority: P0
    suggested_knowledge_level: K3
    expected_value: Ruta de inicio rápido para desarrolladores sin fricción inicial
    materialized_as: AN-073 (candidate)
    dependencies:
      - AN-068
      - AN-070
  - id: INIT-09-07
    title: Aplicaciones de referencia y pruebas E2E
    type: feature
    priority: P1
    suggested_knowledge_level: K3
    expected_value: Dos aplicaciones de referencia (motor de reglas de negocio e integración host)
    materialized_as: AN-074 (candidate)
    dependencies:
      - AN-070
  - id: INIT-09-08
    title: Validación con desarrolladores externos y reporte de fricciones
    type: spike
    priority: P1
    suggested_knowledge_level: K3
    expected_value: Prueba piloto con 5-10 desarrolladores externos con métricas de fricción
    materialized_as: AN-075 (candidate)
    dependencies:
      - AN-073
      - AN-074
  - id: INIT-09-09
    title: Publicación oficial v0.1.0 y cierre de INI-009
    type: chore
    priority: P0
    suggested_knowledge_level: K4
    expected_value: Entrega formal de la versión pública v0.1.0 con artefactos firmados y portal actualizado
    materialized_as: AN-076 (candidate)
    dependencies:
      - AN-069
      - AN-070
      - AN-073
      - AN-075
---

# INI-009 — Public Release & Developer Adoption

**Tipo:** Iniciativa de producto y adopción  
**Estado propuesto:** Draft  
**Prioridad:** Alta  
**Dependencias:** INI-001 a INI-008 completadas  
**Producto:** Wasma / Agent Native  
**Hito objetivo:** Primera versión pública v0.1.0

## 1. Contexto y oportunidad

Wasma completó su roadmap técnico inicial con 67 Work Items y ocho iniciativas entregadas. El proyecto cuenta con un compilador basado en Semantic IR, un runtime WebAssembly independiente, mecanismos de validación y seguridad, herramientas de desarrollo, documentación y evidencia experimental de rendimiento.

La siguiente etapa no debería centrarse exclusivamente en añadir capacidades técnicas. La principal oportunidad consiste en reducir las barreras de adopción y comprobar que personas externas al desarrollo pueden utilizar Wasma para resolver problemas reales.

**Hipótesis de la iniciativa:** si Wasma dispone de una distribución confiable, una experiencia de aprendizaje sencilla y ejemplos prácticos reproducibles, los desarrolladores podrán evaluar sus capacidades sin necesitar asistencia directa del autor.

## 2. Objetivo general

Publicar la primera versión distribuible de Wasma, ofrecer una experiencia de instalación y aprendizaje accesible, y validar su utilidad mediante aplicaciones de referencia y pruebas con desarrolladores externos.

### Objetivos específicos

**Distribución y confianza.** Producir binarios oficiales para las plataformas soportadas, acompañados de mecanismos de verificación criptográfica, trazabilidad, notas de versión y procedimientos de publicación seguros.

**Developer Experience.** Facilitar que una persona sin conocimiento previo de la arquitectura pueda instalar la CLI, comprender los conceptos principales y construir, verificar y ejecutar su primer componente siguiendo la documentación.

**Adopción y validación.** Construir casos de uso representativos, observar la experiencia de usuarios externos e identificar problemas técnicos, funcionales y de documentación para orientar la evolución del producto.

**Identidad y transparencia.** Fortalecer el portal público mediante una sección del autor y metadatos que permitan identificar exactamente qué versión está desplegada.

---

## 3. Alcance de la iniciativa

### Línea A — Public Release v0.1.0

La primera versión pública debe ser una entrega reproducible y verificable, sin presentarla como una certificación general para producción.

Se debe incluir:

- Versionamiento coherente de los crates, CLI, tags y artefactos.
- Binarios para Linux x86_64 GNU, Windows x86_64 MSVC y macOS ARM64.
- Paquetes comprimidos por plataforma y arquitectura.
- Catálogo SHA-256 y firma Minisign con clave oficial.
- Publicación controlada en GitHub Releases.
- Instrucciones de instalación, actualización y verificación.
- Changelog, notas de versión, limitaciones y compatibilidad declarada.
- Pruebas de instalación desde los paquetes finales, en entornos limpios.

La activación de firma y publicación oficial requiere una decisión humana que sustituya el bloqueo existente en el workflow. Las claves privadas nunca deben incorporarse al repositorio.

### Línea B — Developer Experience

La experiencia inicial debe permitir que el desarrollador complete el siguiente recorrido:

**Descubrir Wasma → Instalar → Comprender → Construir → Verificar → Ejecutar → Integrar.**

El portal debe incluir una ruta de inicio rápido para personas que solo desean ejecutar su primer componente, sin obligarlas a conocer previamente Rust, Wasmtime, Semantic IR o WIT.

Se diferenciarán los requisitos del desarrollador que compila el propio proyecto de los del usuario que simplemente instala la CLI distribuida.

También se actualizarán los ejemplos, se eliminarán referencias a estados técnicos históricos que ya no correspondan y se mantendrán visibles las limitaciones experimentales.

### Línea C — Aplicaciones de referencia y validación

Construir inicialmente dos ejemplos representativos:

**Ejemplo 1: Motor de reglas de negocio.** Un componente WebAssembly que evalúe reglas deterministas, como descuentos, elegibilidad o puntuaciones. Debe evidenciar generación, verificación, ejecución y pruebas de aceptación.

**Ejemplo 2: Integración desde una aplicación host.** Una aplicación que consuma un componente Wasma desde el runtime y demuestre aislamiento, límites y contratos definidos. Las dependencias externas, si existen, deberán usar interfaces y capacidades expresamente soportadas; no se prometerá acceso general a red o archivos.

Después se desarrollará una prueba piloto con miembros de la comunidad que no hayan participado en la implementación del proyecto.

Se medirá la facilidad de instalación, el tiempo hasta obtener el primer resultado, la finalización de tareas, los errores encontrados, la claridad documental y los casos de uso sugeridos.

---

## 4. Mejoras del portal web

### Sección «Sobre el autor»

Incorporar una sección de autor en la página principal, idealmente después de explicar las capacidades del producto y antes del cierre de la landing.

**Contenido propuesto:**

- Nombre del creador: Julián Darío Luna Patiño, sujeto a aprobación editorial.
- Breve presentación profesional.
- Motivación y origen de Wasma.
- Relación con la investigación, el desarrollo de software y la creación de herramientas open source.
- Fotografía o recurso visual aprobado, si se proporciona.
- Enlaces verificables a perfiles profesionales, comunidad y repositorios.
- Reconocimiento a quienes contribuyen al proyecto.

La sección debe aportar confianza y contexto sin convertir el producto en una página personal. Puede ampliarse posteriormente con una página `/autor/`.

### Versión desplegada en el footer

Agregar al footer global información generada automáticamente durante el proceso de build.

Ejemplo de presentación:

**Wasma Web v0.1.0 · Build `<SHA corto>` · Ver código fuente**

La implementación deberá distinguir:

- **Versión del sitio:** obtenida de la versión propia de `website/`.
- **Build desplegado:** commit Git exacto utilizado para generar los archivos estáticos.
- **Versión de Wasma CLI:** mostrada separadamente cuando exista una release distribuida, sin inferirla de la versión del sitio.

El SHA deberá enlazar al commit correspondiente. En producción no se permitirá publicar un identificador inventado ni asumir que HEAD de `main` coincide siempre con el deploy vigente.

Los builds locales sin información de despliegue deben identificar explícitamente su carácter de desarrollo. La solución deberá funcionar también con los previews de Netlify y conservar la presentación responsive y accesible.

---

## 5. Work Items propuestos

| ID | Work Item | Prioridad | Dependencias |
|---|---|---|---|
| AN-068 | Auditoría de preparación de release y contrato de versionamiento | P0 | INI-001…008 |
| AN-069 | Firma oficial, integridad y controles del pipeline de publicación | P0 | AN-068 |
| AN-070 | Empaquetado, instalación y prerelease multiplataforma | P0 | AN-069 |
| AN-071 | Sección de autor e identidad editorial del proyecto | P1 | Portal actual |
| AN-072 | Versión, commit y procedencia del despliegue en el footer | P1 | Portal actual |
| AN-073 | Onboarding, quickstart y reconciliación de documentación | P0 | AN-068, AN-070 |
| AN-074 | Aplicaciones de referencia y pruebas E2E | P1 | AN-070 |
| AN-075 | Validación con desarrolladores externos y reporte de fricciones | P1 | AN-073, AN-074 |
| AN-076 | Publicación oficial v0.1.0 y cierre de INI-009 | P0 | AN-069, AN-070, AN-073, AN-075 |

Los identificadores son propuestas nuevas y deben reservarse mediante Kaddo antes de materializar los Work Items.

## 6. Estrategia de ejecución

**Fase 1 — Preparación y transparencia.** Completar la auditoría de distribución (AN-068), incorporar la sección del autor (AN-071), mostrar metadatos reales del despliegue (AN-072) y reconciliar la documentación del sitio con las capacidades existentes.

**Fase 2 — Distribución y experiencia inicial.** Completar firma y empaquetado (AN-069/070), publicar una versión candidata de prueba y validar instalación, quickstart y ejemplos desde paquetes reales (AN-073/074). Esta fase todavía no autoriza la publicación oficial.

**Fase 3 — Validación y publicación.** Invitar inicialmente a entre cinco y diez desarrolladores externos para probar el recorrido sin asistencia directa. Documentar resultados y defectos, resolver los problemas bloqueantes, aprobar la release y publicar v0.1.0 (AN-075/076).

Las tres fases podrán tener tareas en paralelo cuando sus dependencias lo permitan.

---

## 7. Métricas de éxito

| Dimensión | Indicador | Evidencia |
|---|---|---|
| Distribución | Instalación satisfactoria en las 3 plataformas objetivo | Pruebas desde paquetes publicados |
| Seguridad | Hashes y firma oficial verificables | Artefactos y CI de release |
| Developer Experience | Tiempo hasta el primer componente ejecutado | Sesiones piloto |
| Autonomía | Proporción de participantes que completan el tutorial sin ayuda | Observación y registro |
| Calidad documental | Bloqueos, errores y pasos ambiguos identificados y resueltos | Issues y pruebas de documentación |
| Casos prácticos | Dos aplicaciones de referencia reproducibles | Repositorio, tests y tutoriales |
| Transparencia web | Versión y SHA del deploy coinciden con producción | Validación del sitio desplegado |
| Adopción inicial | Participación y feedback de desarrolladores externos | Reporte piloto |

Los objetivos cuantitativos de adopción se establecerán antes del piloto y se contrastarán con los resultados observados. No se atribuirá éxito a descargas, visitas o estrellas de GitHub sin evidencia de utilización.

## 8. Condiciones de cierre

INI-009 podrá cerrarse cuando:

1. La v0.1.0 haya sido publicada con autorización humana, artefactos descargables, integridad y firmas verificables.
2. La instalación y el recorrido inicial funcionen desde los paquetes distribuidos en las plataformas declaradas.
3. Los dos ejemplos de referencia estén documentados y probados.
4. El portal muestre la sección del autor y el identificador exacto de la versión web desplegada.
5. La prueba piloto haya producido evidencia de uso, hallazgos y decisiones de mejora.
6. Se hayan documentado los límites conocidos, sin afirmar compatibilidad universal ni certificación de producción.
7. Se haya aprobado una propuesta de siguiente roadmap basada en los resultados.

## 9. Fuera del alcance

Esta iniciativa no compromete nuevos tipos de Semantic IR, un servicio HTTP persistente, compatibilidad general con cualquier componente WebAssembly, infraestructura de múltiples tenants como servicio administrado, ni integraciones externas todavía no soportadas.

Las solicitudes surgidas durante la validación deberán registrarse y priorizarse. No se incorporarán automáticamente al alcance de la release.

## 10. Resultado esperado

Al finalizar INI-009, Wasma deberá dejar de ser únicamente un proyecto con un núcleo tecnológico completo y convertirse en una herramienta open source distribuible, documentada y evaluada por desarrolladores externos.

El éxito de esta etapa no será simplemente publicar v0.1.0: será demostrar que una persona nueva puede comenzar a utilizar Wasma y entender sus capacidades reales sin depender del equipo que lo construyó.
