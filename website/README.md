# Sitio de Wasma

Astro + Starlight, contenido español y salida estática. No modifica Cargo ni necesita
Rust para generar HTML. Node **24.13.0**, pnpm **12.9.1**; versiones fijadas en lockfile.

## Desarrollo y comprobación

Desde la raíz del repositorio:

```sh
pnpm --dir website install --frozen-lockfile
pnpm --dir website check
pnpm --dir website test
pnpm --dir website build
pnpm --dir website check:links
pnpm --dir website check:content
pnpm --dir website dev
```

Para ver el build: `pnpm --dir website preview --port 4321`. La búsqueda Pagefind se
comprueba sobre el build, no solamente sobre dev. Salida: `website/dist/`.

El tutorial ejecutable requiere una CLI compilada desde el mismo checkout:

```sh
cargo build --locked --release -p agent-native
pnpm --dir website test:docs
```

`AGENT_NATIVE_BIN` permite seleccionar una ruta absoluta alternativa; documenta su
SHA y procedencia. El script extrae fixtures y comandos de la página Markdown, usa un
directorio temporal propio y verifica éxitos y fallo esperado. Nunca ejecuta texto
de un servicio externo. Las pruebas de validadores usan fixtures temporales, sin
modificar las iniciativas reales.

## Contenido y diseño

- `src/content/docs/docs/`: rutas Starlight `/docs/`. Cada página tiene title/description.
- `src/content/blog/`: Markdown/MDX con title, description, date y tags; índice cronológico.
- `src/styles/tokens.css`: paleta y tipografías; `site.css` y `docs.css`: presentaciones.
- `src/components/Brand.astro`: encuadre CSS de imágenes originales, sin redibujar el logo.
- `src/lib/roadmap.mjs`: lee solo cuatro campos escalares públicos de INI-001…008.
  Notas históricas con YAML no válido fuera de esos campos no se publican ni interpretan.
  Estados desconocidos y bloques ausentes fallan el build. Añadir estados exige revisión.
- `src/assets/`: PNG originales aportados por el usuario el 2026-10-20. Astro produce
  WebP optimizado en build. `public/brand/` contiene imagen social y favicon.

Starlight tiene locale raíz español. Para inglés: añadir locale `en`, contenidos
equivalentes y traducciones de navegación; no crear páginas vacías ahora.

## Marca y procedencia

Se recibieron kit, lámina de logo/isotipo (dos adjuntos idénticos) y banner oscuro.
Se preserva una copia de cada imagen distinta. La autorización del usuario cubre su
uso como referencia e imágenes del sitio. No se afirma que los PNG sean originales
vectoriales. El favicon encuadra el isotipo existente en un viewport SVG; no es un
redibujo vectorial. El build reduce su peso usando la imagen optimizada por Astro.

La familia tipográfica exacta no viene identificada: se utiliza una pila de sistema
geométrica/legible, pendiente de revisión visual humana. La variante monocroma está
preservada en la lámina original; no se necesita en las superficies actuales.

## CI

`.github/workflows/website.yml` añade dos jobs: web estática y smoke del tutorial con
Rust. No despliega ni modifica workflows existentes. Un artefacto descargable contiene
`dist`. Evidencia y límites en `validation/README.md`.

## Despliegue propuesto: Netlify (pendiente de elección)

Netlify admite el build estático de Astro sin adaptador cuando no hay funciones de
servidor. Fuente: [Astro en Netlify](https://docs.netlify.com/build/frameworks/framework-setup-guides/astro/).
No se creó cuenta, sitio, integración ni despliegue. No se necesita token para build local.

Tras aprobación del proveedor y del responsable de dominio:

1. Importar el repositorio completo, no solo `website/`: el build lee `knowledge/`.
2. Configurar base `website`, comando `pnpm build`, publish `dist`, Node 24.13.0,
   pnpm 12.9.1. Instalación `pnpm install --frozen-lockfile`. Seleccionar la rama
   de producción aprobada; conservar los checks de CI antes de publicar.
3. Habilitar Deploy Previews para PR aprobadas. Netlify proporciona `CONTEXT=deploy-preview`;
   las páginas emiten `noindex,nofollow`. No publicar previews sin autorización.
4. Añadir `wasma.org` y `www.wasma.org` como dominios; elegir apex como principal.
   Consultar y conservar primero los registros existentes. Aplicar únicamente los
   registros indicados por el panel para este sitio, con autorización del titular.
   No copiar IPs genéricas ni reemplazar MX/TXT existentes.
5. Verificar emisión del certificado HTTPS para ambos nombres y redirección permanente
   www → apex conservando ruta/query. Probar `/docs/first-component/` y `/blog/`.
   Canonical y sitemap de producción apuntan a `https://wasma.org`.
6. Hacer smoke de páginas, 404, assets y búsqueda; guardar ID de deploy y SHA.
   Ante regresión, restaurar el deploy anterior desde el historial y registrar el motivo.

Puede usarse otro hosting estático con esas mismas rutas, 404 y redirección. La elección
operativa y el control del dominio siguen pendientes; esta guía no autoriza DNS ni merge.

Referencias de framework: [Starlight manual](https://starlight.astro.build/manual-setup/),
[Astro Content Collections](https://docs.astro.build/en/guides/content-collections/).
