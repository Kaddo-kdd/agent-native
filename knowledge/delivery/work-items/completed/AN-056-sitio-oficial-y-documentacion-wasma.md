---
type: feature
id: AN-056
title: AN-056 — Sitio oficial y portal de documentación de Wasma
status: completed
completed_at: '2026-10-26'
implemented_by: codex
closed_by: agentic
implementation_status: completed
validation_status: passed
work_type: feature
created_at: '2026-10-20'
source:
  type: external
  imported_at: '2026-10-20'
  source_format: markdown
  source_hash: 473b27a08babddec73c972731502a253ac971f0d052ec8c27e5fe4884ea77fd4
  inferred: false
generated_by: kaddo-admin
affected_modules: []
knowledge_level: K3
suggested_knowledge_level: K3
scope_confidence: medium
project_state: ai-assisted
refined_by: work-item-agent
updated_at: '2026-10-20'
domains:
  - Developer Experience, Documentation & Branding
depends_on:
  - AN-057
  - AN-058
  - AN-059
  - AN-060
summary: AN-056 — Sitio oficial y portal de documentación de Wasma
original_snapshot:
  title: AN-056 — Sitio oficial y portal de documentación de Wasma
---

# AN-056 — Sitio oficial y portal de documentación de Wasma

## Refinamiento — 2026-10-20

Este documento conserva íntegramente el alcance importado debajo de este refinamiento.
AN-056 coordina la aceptación global; la implementación se divide en cuatro Work Items
draft enlazados abajo. La división no elimina entregables ni autoriza implementación.
No se crea una iniciativa ni se asocia a RM-08: la presencia web es transversal.

### Actor, problema y resultado observable

**Actor:** desarrollador que descubre Wasma, intenta ejecutar su primer componente o
consulta contratos de CLI y seguridad; colaborador que mantiene documentación.
**Problema:** el repositorio contiene conocimiento técnico, pero carece de una entrada
pública coherente, documentación navegable y ejemplos editorialmente verificados.
**Estado actual:** no existe `website/`; `assets/` solo contiene
`agent-native-banner.png`, sin kit oficial Wasma. El producto sigue usando `agent-native`.
**Resultado:** desde `/`, el visitante entiende el producto, accede a `/docs/`, ejecuta
un ejemplo reproducible y distingue capacidades disponibles, experimentales y futuras.
El mantenedor añade contenido con Markdown/MDX y valida un build estático independiente.

**Recorrido E2E:** landing → CTA de documentación → instalación → primer componente
con archivos completos y resultado esperado → consulta CLI/seguridad → roadmap o
benchmarks → evidencia canónica en GitHub. Búsqueda, enlaces directos, móvil, teclado
y tema claro/oscuro deben conservar este recorrido. No intervienen API ni base de datos;
solo navegación y preferencias de interfaz. No se recogen datos de usuario.

### Evidencia y discrepancias que condicionan el contenido

| Fuente local inspeccionada | Hallazgo y consecuencia |
|---|---|
| `crates/cli/src/cli.rs` | `Commands` contiene `build`, `verify`, `test`, `run`, `publish`; no hay subcomando `spec`. `build` recibe `input` posicional y `--output`; `verify` recibe `.md`/`.json`. La integridad binaria pertenece a `run --verify`. |
| `README.md` y `knowledge/tech/cli-guide.md` | Hay ejemplos y afirmaciones desalineados con el parser. No copiarlos sin ejecutar; no añadir aliases al producto para hacerlos funcionar. |
| `crates/cli/tests/cli_e2e.rs` | Fuente de fixtures y flujo offline para preparar un tutorial verificable; el smoke del tutorial debe ejecutar exactamente lo publicado. |
| `knowledge/delivery/roadmap.md`, `knowledge/delivery/initiatives/INI-001`…`INI-008` | El roadmap conserva propuestas históricas. Usar estados estructurados de las iniciativas para los bloques y los WIs para sus límites; no inferir disponibilidad de toda capacidad por un hito global. |
| AN-050 en `ready/`, INI-007 en `completed/` | El baseline WASI completado no acredita soporte general de binarios reales `wasm32-wasip2`; esa ampliación sigue pendiente. |
| `docs/benchmarks/README.md`, `publication.json`, AN-055 | Publicación candidata; ninguna fila certificada. La web puede enlazarla sin depender del cierre de AN-055 ni de RM-08. |
| `Cargo.toml`, `.github/workflows/{ci,release,bench}.yml` | Cuatro crates y workflows existentes; sitio y CI web se añaden sin alterar sus contratos. |
| `assets/` | No se encontraron logo/isotipo oficiales Wasma; no sustituirlos por una recreación ni dar AC-02 por cumplido. |

El gate `kaddo questions --json` reportó 0 preguntas bloqueantes abiertas y 10
importantes abiertas. Esto no acredita readiness de AN-056: las preguntas específicas
de marca y entrega se registran abajo. No se resuelven preguntas arquitectónicas
históricas mediante contenido comercial.

### Impact analysis y Module coverage

| Superficie / módulo | Evaluación | Trabajo o límite |
|---|---|---|
| Producto/UI, frontend, contenido, documentación | affected | Landing, Starlight, blog, roadmap y enlaces de comunidad. |
| Configuración y dependencias web | affected | TypeScript, pnpm, versiones compatibles fijadas al implementar, lockfile, scripts y site URL. |
| Operaciones / release | affected | Build, verificación y guía; hosting efectivo pendiente de elección, DNS/publicación fuera de ejecución autorizada. |
| `website/` (nuevo directorio, no módulo Kaddo registrado) | affected | Única aplicación nueva; sin React, backend ni CMS. |
| `knowledge/`, `docs/benchmarks/` | affected | Fuentes editoriales y de estados; lectura selectiva, sin publicar WIs/ADRs internos en bloque. |
| `crates/{ir,compiler,runtime,cli}` | reviewed-not-affected | Fuentes de verificación; sin cambios funcionales o dependencias nuevas. |
| Backend, base de datos, autenticación, notificaciones, analytics | not-applicable | No se incorporan en esta entrega. |
| Feature flags | reviewed-not-affected | Solo explicar el flag existente `--experimental`; no crear flags del producto. |
| Recursos externos / proveedor de hosting | unknown | Dominio declarado `wasma.org`; no se verificó control operativo ni proveedor. No existen recursos registrados que justifiquen inventar IDs `resources`. |

K3 se mantiene: hay alcance y fuentes concretas, pero faltan validaciones de contenido,
kit y decisiones operativas. Confianza **medium**: trabajo web acotado con incertidumbre
de branding y discrepancias editoriales conocidas, sin cambios al runtime.

### División y trazabilidad de aceptación

| Work Item | Resultado | Dependencias | AC originales cubiertos |
|---|---|---|---|
| [AN-057](AN-057-wasma-web-foundation.md) | Fundación Astro/Starlight, branding, landing y temas | Kit oficial para aceptación visual | 01, 02, 03, 08, 09, 13 |
| [AN-058](AN-058-wasma-user-documentation.md) | Documentación inicial y tutorial ejecutable | AN-057 | 04, 07, 10 |
| [AN-059](AN-059-wasma-blog-roadmap.md) | Blog, roadmap trazable y enlaces de comunidad | AN-057; AN-058 para revisión integrada | 05, 06, 07 |
| [AN-060](AN-060-wasma-web-validation-delivery.md) | CI, validación integral y preparación de despliegue | AN-057, AN-058, AN-059 | 09, 11, 12, 14, 15; regresión 01–13 |

AN-056 no es una tarea monolítica para pasar directamente a implementación. Cada hijo
tiene su propio alcance y aceptación. Los AC originales son obligatorios para el cierre
global; no basta con que los hijos compilen aisladamente.

### Decisiones de refinamiento y límites

- [resolved] Español inicial, Astro + Starlight, pnpm, salida estática, rutas de la
  propuesta y continuidad del nombre técnico: son requisitos explícitos importados.
- [assumed] La primera entrega de rendimiento será una página contextual con enlace a
  la evidencia, sin tablas numéricas duplicadas. Si se muestran cifras, deberán generarse
  desde `publication.json` y conservar advertencias y procedencia.
- [assumed] El roadmap público derivará estados durante el build de los frontmatters
  locales INI-001…INI-008, con links a fuentes y límites de WIs pendientes. No habrá un
  JSON manual de estados paralelo. Un estado desconocido o una fuente ausente debe hacer
  fallar la validación, no convertirse en “completado”.
- [assumed] El tutorial inicial será offline (`synthesizer` o fixture explícito), sin
  exigir claves LLM. Explicará que este modo no interpreta libremente cualquier narrativa.
- [resolved] Publicación, DNS, HTTPS efectivo y previews alojadas solo se activan tras
  autorización operativa. Preparar instrucciones no equivale a desplegar.
- [deferred] Migración técnica del nombre, capacidades futuras, certificación de
  benchmarks y elección de interfaz pública estable del runtime: otros trabajos.

### Open Questions

| ID / estado | Pregunta y siguiente evidencia | Efecto |
|---|---|---|
| WEB-01 [open] | ¿Dónde están los originales aprobados del logo, isotipo y variantes claro/oscuro/monocromo? Registrar origen y permisos de uso al recibirlos. | Bloquea AC-02 y aceptación visual de AN-057; no impide refinar ni preparar estructura. |
| WEB-02 [open] | ¿Cuál es la familia tipográfica oficial y su licencia? Proponer una pila provisional explícita si el kit no la define. | Bloquea aprobación final de branding, no el build técnico. |
| WEB-03 [open] | ¿Qué proveedor y responsable administrarán hosting y dominio? AN-060 presentará una propuesta verificando documentación vigente. | No bloquea sitio local; la preparación específica del proveedor se completa tras selección. Publicación requiere autorización separada. |
| WEB-04 [open] | ¿Qué binario/release será la referencia del tutorial? Elegir al implementar y registrar versión/SHA; contrastar con el código fuente. | Bloquea dar ejemplos por verificados; instalación desde fuente es una alternativa documentable. |

### How to test it (validation) y Definition of Done

Estos son contratos de validación a implementar, **no comandos ejecutados en este refinamiento**:

1. Desde checkout limpio: `pnpm --dir website install --frozen-lockfile`,
   `pnpm --dir website check`, `pnpm --dir website build`,
   `pnpm --dir website check:links` y `pnpm --dir website check:content`.
   `check` cubre TypeScript/Astro; links cubre rutas, assets y fragmentos del HTML generado;
   content comprueba fuentes de roadmap, metadatos y procedencia de ejemplos.
2. `pnpm --dir website test:docs` ejecuta el tutorial publicado contra la CLI seleccionada,
   sin LLM/red en el flujo del ejemplo; comprueba resultado esperado y fallo de un caso
   inválido documentado. Herramientas Rust solo en este job, no en el build web.
3. Servidor local y preview del build: navegar todas las rutas en 375, 768 y 1440 px;
   probar teclado, foco visible, búsqueda, copiar comandos, 404, temas y ausencia de
   scroll horizontal global. Revisión de contraste WCAG AA (4.5:1 texto normal y 3:1
   texto grande), texto alternativo y reduced motion. Registrar capturas y hallazgos.
4. Recorrido E2E descrito arriba completado con un visitante nuevo, desde instalación
   documentada hasta salida del primer componente, y enlace a evidencia de seguridad.
5. Revisar title, description, canonical y Open Graph por página, sitemap y enlaces de
   edición; no indexar previews públicas cuando se habiliten.
6. Conservar logs, versión de herramientas, SHA de fuentes y evidencia visual en
   `website/validation/` (ruta propuesta). No registrar una aprobación inexistente.

**Cierre:** todos los AC originales y de los hijos satisfechos; instrucciones reproducibles;
revisión humana registrada con fecha, alcance y observaciones resueltas. Mantener las
fases A–D originales. La CI remota se ejecuta solo después del push autorizado; hasta
entonces se informa como pendiente y se conservan resultados locales equivalentes.
Un bloqueo operativo de publicación puede quedar abierto si el sitio está técnicamente
preparado y se documenta responsable/acción pendiente. No se cierra con branding provisional.

**Ownership sugerido:** `website/**`, `.github/workflows/website.yml`; cambios editoriales
canónicos solo si resultan necesarios y con alcance explícito. No se asignan globs Rust.
**Handoff:** revisar los borradores y resolver sus preguntas antes de solicitar `ready`.
La skill de refinamiento no concede readiness; el cambio de estado requiere confirmación
humana. No se ha implementado el sitio ni realizado commit, push o despliegue.

## Definición importada (preservada)

## Metadata

- **ID:** AN-056 (propuesto)
- **Tipo:** feature
- **Estado inicial:** draft
- **Nivel de conocimiento:** K3
- **Título:** Implementar el sitio oficial de Wasma con Astro y Starlight
- **Dominio:** Developer Experience, Documentation & Branding
- **Iniciativa:** Nueva iniciativa transversal de presencia web y documentación pública
- **Dependencias:** Identidad visual Wasma, CLI y documentación técnica existentes
- **Dominio web:** `wasma.org`
- **Repositorio:** `Kaddo-kdd/agent-native` (nombre técnico actual)

## 1. Contexto

Agent Native ha evolucionado hasta convertirse en un proyecto con capacidades funcionales de generación, verificación, compilación y ejecución de componentes WebAssembly.

El producto adopta la identidad pública **Wasma**, con dominio oficial `wasma.org` y un kit visual que incluye logotipo, isotipo, versiones monocromáticas, paleta cromática y elementos gráficos.

Actualmente, el repositorio concentra la documentación en `knowledge/`, el README y documentos técnicos como `docs/benchmarks/`. Esta información es valiosa para el desarrollo, pero no constituye todavía una experiencia de documentación orientada a usuarios externos.

Se requiere un sitio web que permita presentar Wasma, explicar sus conceptos, facilitar los primeros pasos y acompañar la evolución del producto sin esperar a completar RM-08.

La adopción de la marca pública no implica renombrar inmediatamente el ejecutable, los crates Rust ni el repositorio.

## 2. Objetivo

Construir la primera versión del sitio oficial de Wasma utilizando **Astro + Starlight**, integrando su identidad visual y proporcionando una plataforma extensible para documentación técnica, ejemplos, blog, roadmap y comunidad.

El sitio debe ser minimalista, moderno, rápido, accesible y fácil de mantener mediante Markdown y MDX.

**Resultado esperado:** una aplicación web funcional, ejecutable localmente y preparada para desplegarse en `wasma.org`, sin introducir dependencias en los crates Rust del producto.

## 3. Alcance funcional

### 3.1. Página principal — `/`

Desarrollar una landing personalizada con Astro, utilizando el kit de marca de Wasma.

Debe incluir:

- Encabezado con logotipo oficial y navegación.
- Hero con el mensaje principal: **From spec to wasm.**
- Descripción clara del propósito del producto: transformar especificaciones funcionales en componentes WebAssembly verificables y ejecutables.
- Explicación visual del flujo Specification → Semantic IR → Verification → Compilation → WebAssembly → Runtime.
- Sección de capacidades actuales: compilación, ejecución independiente, seguridad, CLI, pruebas y evolución de especificaciones.
- Ejemplo técnico breve con comandos reales de la versión disponible.
- Enlaces a documentación, repositorio GitHub, releases y roadmap.
- Footer con información del proyecto, licencia MIT y enlaces relevantes.

No presentar las funcionalidades planeadas como implementadas.

### 3.2. Portal de documentación — `/docs/`

Implementar Starlight dentro de Astro, con navegación organizada y personalización visual acorde a Wasma.

**Estructura inicial de contenido:**

| Sección | Contenido |
|---|---|
| Introducción | Qué es Wasma, problema que resuelve y arquitectura general |
| Getting Started | Requisitos, instalación actual, primer componente |
| Core Concepts | SystemSpec, Semantic IR, WIT, WASM, Component Model |
| CLI | Comandos disponibles, opciones y ejemplos |
| Runtime | Ejecución, ciclo de vida, aislamiento y recursos |
| Security | Políticas, permisos, fuel, memoria e integridad |
| Examples | Ejemplos reproducibles |
| Performance | Metodología, resultados candidatos y limitaciones |

**Contenido mínimo de la primera entrega:** introducción, instalación, primer ejemplo reproducible, conceptos principales, CLI y seguridad.

Cada página debe incluir título, descripción, navegación coherente y ejemplos comprobados cuando corresponda.

### 3.3. Blog — `/blog/`

Crear una sección para comunicar novedades del producto, avances de desarrollo, decisiones arquitectónicas y análisis técnicos.

Requisitos:

- Entradas escritas en Markdown/MDX mediante Astro Content Collections.
- Página índice con fecha, título, resumen y etiquetas.
- Página individual por artículo.
- Ordenación cronológica y metadatos SEO.
- Posibilidad de incorporar imágenes, código y diagramas.

Incluir al menos un artículo inicial sobre el origen, propósito y estado de Wasma. El blog no necesita CMS, base de datos ni autenticación en esta etapa.

### 3.4. Roadmap — `/roadmap/`

Crear una página que explique los hitos RM-01 a RM-08 y el estado general del proyecto.

Mostrar claramente:

- Hitos completados.
- Trabajo en progreso.
- Capacidades planeadas.
- Características experimentales.
- Enlace al roadmap técnico del repositorio.

La información debe provenir de una fuente mantenible y verificable, sin crear un segundo roadmap independiente que pueda quedar desactualizado.

### 3.5. Comunidad y código abierto

Incorporar enlaces visibles al repositorio GitHub, documentación de contribuciones, licencia, issues y releases.

No construir funcionalidades sociales propias en esta primera versión.

## 4. Identidad visual

Aplicar la identidad oficial suministrada para Wasma.

### 4.1. Elementos de marca

- Logotipo principal: isotipo de llama/núcleo y nombre **wasma**.
- Isotipo independiente para favicon y espacios reducidos.
- Versiones sobre fondo claro y oscuro.
- Versión monocromática para contextos que la requieran.
- Diseño visual inspirado en el material de referencia del kit.

### 4.2. Paleta base

| Color | HEX | Aplicación |
|---|---|---|
| Obsidiana | `#0A080F` | Fondo oscuro y texto principal |
| Hueso | `#EDECE8` | Fondos claros y superficies |
| Plasma violeta | `#A856FF` | Acentos principales |
| Plasma magenta | `#FF209E` | Estados y acentos secundarios |
| Plasma cian | `#2FD6FF` | Elementos tecnológicos y enlaces destacados |
| Alquimia oro | `#F6C445` | Detalles especiales y verificación |

Los colores deberán consolidarse en design tokens reutilizables mediante variables CSS.

### 4.3. Principios de diseño

- Minimalismo y jerarquía visual clara.
- Tipografía sans serif geométrica, moderna y legible.
- Código presentado con tipografía monoespaciada.
- Uso moderado de violeta y cian.
- Contraste adecuado en temas claro y oscuro.
- Componentes visuales consistentes.
- Animaciones sutiles, sin perjudicar rendimiento ni accesibilidad.
- Diseño responsivo para escritorio, tablet y móvil.

El estilo visual debe reflejar una herramienta tecnológica seria, moderna y accesible, sin convertirse en una landing genérica de SaaS.

Los archivos de imagen compartidos sirven como referencia visual. Para producción deben utilizarse recursos originales de marca con calidad suficiente, preferiblemente SVG para logo e isotipo. No redibujar ni alterar arbitrariamente la identidad.

La familia tipográfica exacta queda pendiente de validación con los archivos de marca originales.

## 5. Arquitectura técnica

### 5.1. Stack

- **Framework:** Astro.
- **Documentación:** Starlight.
- **Lenguaje:** TypeScript.
- **Contenido:** Markdown y MDX.
- **Blog:** Astro Content Collections.
- **Estilos:** CSS y design tokens.
- **Renderizado:** estático, salvo necesidad justificada.
- **Gestión de dependencias:** pnpm, con lockfile versionado.
- **Hosting:** proveedor de despliegue estático a definir y aprobar.

Evitar incorporar React, bases de datos, CMS externos o servicios backend sin una justificación funcional.

### 5.2. Organización del proyecto

Crear el sitio en el mismo repositorio:

```text
agent-native/
├── crates/
├── knowledge/
├── docs/
│   └── benchmarks/
├── website/
│   ├── public/
│   │   └── brand/
│   ├── src/
│   │   ├── components/
│   │   ├── layouts/
│   │   ├── pages/
│   │   │   ├── index.astro
│   │   │   ├── roadmap.astro
│   │   │   └── blog/
│   │   ├── content/
│   │   │   ├── docs/
│   │   │   │   └── docs/
│   │   │   └── blog/
│   │   └── styles/
│   ├── astro.config.mjs
│   ├── package.json
│   └── pnpm-lock.yaml
└── .github/workflows/
```

Organizar las páginas de Starlight bajo el prefijo `/docs/`, evitando conflictos con la página principal personalizada y las rutas del blog.

La configuración debe permitir añadir nuevos contenidos y secciones sin rediseñar toda la aplicación.

### 5.3. Separación entre conocimiento y documentación

`knowledge/` seguirá siendo la fuente de conocimiento gestionada por Kaddo.

`website/src/content/docs/` contendrá documentación orientada al usuario final.

No copiar indiscriminadamente los Work Items, ADR, documentos internos o registros de decisiones. El contenido público debe ser seleccionado, redactado y verificado para su audiencia.

Los resultados de `docs/benchmarks/` deben mantenerse como fuente técnica canónica. La web podrá mostrar un resumen y enlazar la evidencia completa, sin mantener cifras duplicadas manualmente.

## 6. Transición de Agent Native a Wasma

El sitio utilizará **Wasma** como nombre público principal.

Durante esta etapa:

- No renombrar crates Rust.
- No renombrar el ejecutable `agent-native`.
- No modificar nombres de variables de entorno ni contratos públicos.
- No cambiar la URL del repositorio.
- No modificar ADR históricos por motivos de branding.
- No inventar comandos como `wasma init` o `wasma run` mientras no existan.

Los ejemplos deberán utilizar los nombres técnicos reales.

Incluir una nota discreta en la documentación indicando que Wasma todavía utiliza el ejecutable `agent-native` y que el cambio de nombre técnico se realizará posteriormente.

La migración completa del nombre tendrá su propio Work Item.

## 7. Experiencia de documentación

Implementar las capacidades nativas de Starlight que correspondan:

- Búsqueda de documentación.
- Navegación lateral por categorías.
- Tabla de contenidos.
- Resaltado de sintaxis para código.
- Botón para copiar comandos.
- Selector de tema claro/oscuro.
- Enlaces de edición hacia GitHub.
- Navegación entre documentos.
- Metadatos SEO y sitemap.

El idioma inicial será español, con una estructura preparada para incorporar inglés posteriormente sin obligar a traducir todo el contenido en esta primera entrega.

## 8. Calidad y validación

El sitio debe cumplir los siguientes requisitos:

- Compilación estática exitosa.
- Ausencia de errores de TypeScript y dependencias faltantes.
- Navegación sin enlaces internos rotos.
- Buen contraste visual y navegación por teclado.
- Imágenes con textos alternativos apropiados.
- Diseño usable en pantallas pequeñas.
- Recursos optimizados y carga rápida.
- SEO básico: title, description, canonical, Open Graph y sitemap.
- Sin secretos ni credenciales expuestas.
- Dependencias aisladas del workspace Rust.

Configurar verificaciones de instalación, build y enlaces en CI, sin sustituir ni debilitar los workflows existentes del runtime.

## 9. Despliegue

Preparar el sitio para su publicación en `https://wasma.org`.

El alcance incluye:

1. Documentar el proceso de despliegue.
2. Proponer un proveedor de hosting estático compatible con Astro.
3. Configurar el build de producción.
4. Habilitar HTTPS y dominio personalizado cuando se autorice.
5. Configurar redirección canónica entre dominio raíz y `www`.
6. Preparar previews de cambios para futuras contribuciones.

La configuración de DNS y publicación pública requiere aprobación explícita del responsable del dominio.

No modificar registros DNS existentes sin autorización.

## 10. Criterios de aceptación

**AC-01 — Estructura del proyecto:** existe `website/` con Astro y Starlight integrados y dependencias aisladas del workspace Rust.

**AC-02 — Identidad:** la landing utiliza el logotipo oficial, isotipo, colores y principios visuales suministrados.

**AC-03 — Landing funcional:** la página principal presenta claramente el producto, sus capacidades y enlaces a documentación y GitHub.

**AC-04 — Documentación:** `/docs/` funciona con Starlight y contiene como mínimo las seis páginas esenciales definidas en el alcance.

**AC-05 — Blog:** `/blog/` contiene un índice funcional y una entrada inicial de ejemplo real, sin contenido ficticio presentado como noticia del proyecto.

**AC-06 — Roadmap:** `/roadmap/` diferencia capacidades completadas, experimentales y pendientes.

**AC-07 — Exactitud técnica:** todos los comandos mostrados corresponden a funcionalidades implementadas y verificadas. No se publican comandos futuros como disponibles.

**AC-08 — Temas:** tema claro y oscuro utilizables, con contraste y adaptación de los elementos de marca.

**AC-09 — Responsive:** landing, documentación, blog y navegación funcionan correctamente en móvil, tablet y escritorio.

**AC-10 — Benchmarks:** existe una ruta accesible a la documentación de rendimiento, con advertencias sobre mediciones no certificadas y enlace a la fuente canónica.

**AC-11 — CI:** las verificaciones de compilación y calidad del sitio pasan sin afectar los pipelines Rust actuales.

**AC-12 — Despliegue:** el proyecto queda preparado para publicar en `wasma.org`, con instrucciones verificables y sin cambios DNS no autorizados.

**AC-13 — Mantenibilidad:** colores, tipografía y componentes comunes están centralizados, y las secciones pueden crecer mediante contenido Markdown/MDX.

**AC-14 — Revisión visual local:** la implementación completa se visualiza en un servidor local antes de solicitar autorización de commit.

**AC-15 — Aprobación humana:** no se realiza commit, push, merge ni publicación pública sin la autorización explícita requerida.

## 11. Procedimiento obligatorio de implementación y revisión

La implementación debe avanzar en una rama de trabajo o workspace aislado, sin modificar directamente la rama principal.

**Fase A — Construcción**

Implementar las páginas, componentes, contenido y recursos de marca. Ejecutar las validaciones técnicas y corregir errores.

**Fase B — Visualización local obligatoria**

Levantar el servidor de desarrollo de Astro y verificar visualmente:

- Landing en escritorio y móvil.
- Navegación de documentación.
- Página de blog.
- Tema claro.
- Tema oscuro.
- Aplicación del branding.
- Responsividad y legibilidad.

Presentar capturas o una demostración local, junto con un resumen de los cambios.

**Fase C — Revisión humana**

Solicitar expresamente la aprobación de la apariencia, navegación, contenidos y comportamiento del sitio.

Si existen observaciones, ajustarlas y repetir la revisión local.

**Fase D — Commit y CI**

Solo después de recibir autorización explícita:

- Crear el commit.
- Publicar la rama si se autoriza.
- Abrir el PR.
- Ejecutar CI.
- Revisar el resultado de la integración.

La aprobación del diseño local no autoriza automáticamente el merge ni el despliegue público.

## 12. Fuera de alcance

- Renombrar crates, ejecutable o APIs de Agent Native.
- Migrar el repositorio GitHub.
- Crear un CMS propio.
- Implementar autenticación o cuentas de usuario.
- Desarrollar dashboards o servicios backend.
- Modificar el runtime o el compilador.
- Certificar benchmarks pendientes de revisión.
- Implementar funcionalidades planeadas del roadmap.
- Construir documentación completa para capacidades futuras.
- Publicar releases oficiales o configurar firma de binarios.
- Implementar un sistema propio de analíticas o consentimiento.

## 13. Entregables

1. Proyecto Astro + Starlight en `website/`.
2. Landing oficial de Wasma.
3. Design tokens y componentes de interfaz reutilizables.
4. Logotipo e isotipo correctamente integrados.
5. Documentación inicial funcional.
6. Blog y artículo introductorio.
7. Vista de roadmap.
8. Acceso a evidencia de benchmarks.
9. Scripts de desarrollo, validación y build.
10. Configuración de CI para el sitio.
11. Documentación de instalación y despliegue.
12. Evidencia de revisión visual local y aprobación previa al commit.

## 14. Condición de cierre

AN-056 podrá considerarse completado cuando el sitio cumpla los criterios funcionales y visuales, la documentación inicial esté verificada, el build de producción sea correcto y exista aprobación humana de la implementación.

La publicación en `wasma.org` puede quedar condicionada a una autorización operativa independiente, siempre que el sitio esté técnicamente listo y el bloqueo quede documentado.

**El objetivo de AN-056 es construir la presencia pública y la documentación inicial de Wasma, no completar la transición técnica de nombre ni el roadmap RM-08.**


## Implementación autorizada — 2026-10-20

El usuario autorizó implementación, commit y push y suministró referencias de marca.
Sitio implementado en website/. Evidencia: website/validation/README.md.
Los PNG recibidos resuelven la referencia visual; fuente exacta y aceptación estética
siguen pendientes. Propuesta de hosting documentada, sin DNS ni despliegue.
Estado in-progress hasta revisión humana; no equivale a cierre ni merge autorizado.

## Evidencia de cierre y aceptación global — 2026-10-26

- Los cuatro Work Items dependientes (AN-057, AN-058, AN-059 y AN-060) han sido verificados individualmente y trasladados a `completed/`:
  - **AN-057:** Fundación web, tokens, landing y aprobación de identidad visual.
  - **AN-058:** Documentación completa de 8 páginas y tutorial ejecutable contrastado contra la CLI real (`test-docs.mjs`: 5/5 comandos OK).
  - **AN-059:** Blog y roadmap dinámico verificado contra las 8 iniciativas canónicas (`checks.test.mjs`: 2/2 tests OK).
  - **AN-060:** Scripts de validación, compilación estática (13 páginas en `website/dist/`), resolución total de links y configuración Netlify.
- Validación local interactiva en preview (`http://127.0.0.1:4321/`) con aprobación humana registrada.
- Cierre formal del epic coordinador AN-056 cumplido.
