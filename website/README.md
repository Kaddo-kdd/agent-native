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

## Despliegue en Netlify

Netlify admite el build estático de Astro sin adaptador cuando no hay funciones de
servidor. Fuente: [Astro en Netlify](https://docs.netlify.com/build/frameworks/framework-setup-guides/astro/).
La configuración está en `netlify.toml`, en la raíz del repositorio. Define base
`website`, salida `dist`, Node 24.13.0 e instalación con lockfile congelado. Netlify
selecciona pnpm 12.9.1 desde `packageManager` en `website/package.json`.
El comando comprueba tipos, genera el sitio y valida enlaces y contenido antes de
publicar. Los assets con hash de `/_astro/` tienen caché inmutable de un año.

El build se ejecuta en cada cambio del repositorio (`ignore = "exit 1"`) para incluir
las iniciativas de `knowledge/`, fuera de la base `website`. No requiere Rust ni
variables secretas. Las rutas estáticas y `404.html` se sirven directamente.

Para importar y desplegar:

1. Importar el repositorio completo, no solo `website/`: el build lee `knowledge/`.
2. Seleccionar `main` como rama de producción. Netlify lee `netlify.toml` de la raíz;
   conservar la base `website` y publish `dist` relativos a esa base. Si el asistente
   solicita Package directory, usar `website`. No duplicar `website` en publish.
3. Si se habilitan Deploy Previews, Netlify proporciona `CONTEXT=deploy-preview`;
   las páginas emiten `noindex,nofollow` y conservan los canonical de producción.
4. Añadir `wasma.org` y `www.wasma.org` como dominios; elegir apex como principal.
   Consultar y conservar primero los registros existentes. Aplicar únicamente los
   registros indicados por el panel para este sitio, con autorización del titular.
   No copiar IPs genéricas ni reemplazar MX/TXT existentes.
5. Verificar emisión del certificado HTTPS para ambos nombres y redirección permanente
   www → apex conservando ruta/query. Probar `/docs/first-component/` y `/blog/`.
   Canonical y sitemap de producción apuntan a `https://wasma.org`.
6. Hacer smoke de páginas, 404, assets y búsqueda; guardar ID de deploy y SHA.
   Ante regresión, restaurar el deploy anterior desde el historial y registrar el motivo.

La cuenta, la importación del proyecto y el dominio se configuran en el panel de
Netlify. El commit de configuración no crea por sí solo un proyecto ni modifica DNS.

Referencias de configuración: [netlify.toml](https://docs.netlify.com/build/configure-builds/file-based-configuration/),
[dependencias y pnpm](https://docs.netlify.com/build/configure-builds/manage-dependencies/).

Referencias de framework: [Starlight manual](https://starlight.astro.build/manual-setup/),
[Astro Content Collections](https://docs.astro.build/en/guides/content-collections/).
