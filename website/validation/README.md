# AN-056 — Evidencia de implementación

Fecha: 2026-10-20. Rama: `codex/wasma-website`.
Autorización humana registrada en el chat: implementar, commit y push usando las
imágenes de referencia suministradas. No incluye merge, DNS ni hosting público.

## Validación local realizada

| Comprobación | Resultado |
|---|---|
| Instalación con lockfile | Correcta, pnpm 12.9.1 / Node 24.13.0 |
| Astro/TypeScript | 17 archivos, 0 errores, 0 warnings, 0 hints |
| Build estático | 13 páginas; Pagefind y sitemap generados |
| Rutas, assets, fragmentos internos | Todos resuelven |
| Contenido | Ocho bloques canónicos, rutas mínimas, SEO, idioma y alt verificados |
| Pruebas de validadores | 2 pasan: links rotos/fragmentos y estados/fuentes de roadmap |
| Tutorial publicado | verify/build/run/test con salida 0; función ausente con salida 1 |
| Salida JSON de invocación | success=true, function_name=identity, results=[42] |
| Navegador | Landing, docs, blog/artículo y roadmap revisados; temas claro/oscuro |
| Responsive | Cuatro rutas a 375/768/1440 px, sin overflow horizontal global; responsive.json |
| Interacciones | Menú Enter/Escape, búsqueda “instalación” (3 resultados), copia “¡Copiado!” |
| Consola | Sin errores capturados en las páginas revisadas |

El navegador integrado perdió temporalmente sincronización durante el primer intento
de copia. Se reabrió una pestaña y se verificó correctamente con el control accesible.
No se cambió el comportamiento de Starlight para ocultar ese fallo de herramienta.

## Procedencia del ejecutable local

Este equipo no tiene Cargo en PATH. El smoke local usó un artefacto del repositorio,
no un binario de una fuente externa desconocida:

- Workflow: https://github.com/Kaddo-kdd/agent-native/actions/runs/39458330325
- Artefacto: `package-x86_64-pc-windows-msvc` (ID 12282465284).
- SHA fuente: `ad53c91acb8a8d74a52e997ac9031dc237e0593f`.
- Paquete: `agent-native-v0.0.8-benchmarks-x86_64-pc-windows-msvc`.
- SHA-256 ejecutable: `5e6c295eba5611a787c29d9d2bcf80cf25836910e6438dbbafc3c70e8bb4ae04`.

La CI del sitio compila desde el checkout actual en Linux y vuelve a ejecutar los
bloques Markdown. El binario local y los archivos temporales no se versionan.

## Matriz de aceptación

| AC AN-056 | Evidencia / estado |
|---|---|
| 01 estructura | website/, package.json, lockfile; Cargo sin cambios |
| 02 identidad | Tres PNG distintos preservados; logo por encuadre CSS, favicon sin redibujo; revisión humana pendiente |
| 03 landing | Producto, flujo, capacidades, ejemplo y enlaces visibles |
| 04 documentación | Ocho páginas Starlight y tutorial ejecutado |
| 05 blog | Índice y artículo introductorio fechado, sin release ficticia |
| 06 roadmap | Estados leídos de INI-001…008, límites experimentales/pendientes explícitos |
| 07 exactitud | Parser CLI contrastado, smoke desde Markdown; no aliases inventados |
| 08 temas | Claro y oscuro comprobados; variante oscura del logo, gradiente claro oscurecido para legibilidad |
| 09 responsive | responsive.json y capturas locales |
| 10 benchmarks | Página contextual sin cifras duplicadas, advertencia de candidato |
| 11 CI | Workflow independiente; resultado remoto se informa en el PR |
| 12 despliegue | Guía y propuesta Netlify; selección operativa y DNS pendientes |
| 13 mantenibilidad | Tokens, layouts, colecciones, scripts y formateador |
| 14 revisión local | Preview real y capturas, anteriores al commit de implementación |
| 15 aprobación | Commit/push autorizados expresamente; estética final y merge no aprobados |

## Peso de imágenes y capturas

Astro genera variantes WebP del banner de 13.3, 23.3 y 37.5 KiB, y lockup de 7.2 KiB;
el logo oscuro reutiliza una variante de 10.9 KiB. El favicon de producción empaqueta
el raster optimizado. El PNG social completo se reserva a Open Graph.
Esto es medición de archivos generados, no puntuación Lighthouse ni SLA.

Capturas locales en `screenshots/` (ignoradas por Git): home-desktop-dark,
home-mobile-dark, home-desktop-light, docs-desktop-dark, search-mobile,
blog-tablet-dark, roadmap-mobile-dark y matriz de tamaños en tema claro.

## Pendiente de aceptación humana

- Revisar apariencia, contenidos, navegación y tipografía de sistema elegida.
- Los recursos recibidos son PNG; no se afirma disponer de originales vectoriales.
- Aprobar proveedor y responsable operativo antes de cualquier publicación.
- Los enlaces GitHub de edición apuntan a main y serán útiles tras merge; esta rama
  no cambia main. No se realizó auditoría de accesibilidad certificada ni prueba con
  lectores de pantalla. Reduced motion se contempla en CSS.

AN-056…060 permanecen en progreso hasta revisión humana. La autorización de guardar
cambios no se registra como aprobación estética ni como certificación de benchmarks.
