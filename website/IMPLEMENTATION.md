# AN-056 — Plan de implementación

Autorización humana: 2026-10-20, implementar, commit y push, con imágenes de referencia.
Se conserva el gate de revisión visual humana para cierre; no se autoriza merge ni DNS.

## Enfoque y motivo

Astro estático con Starlight bajo `/docs/`, colección de blog y páginas Astro propias.
CSS compartido, sin framework de cliente. Estado del roadmap leído de iniciativas
canónicas al compilar. Tutorial extraído de bloques ejecutables de la propia página.
Se conservan los PNG suministrados; la presentación CSS puede encuadrar el logo sin
redibujarlo. Tipografía de sistema provisional: el kit no identifica una familia.

Se descartan CMS y copia manual del estado del roadmap por coste de mantenimiento.
El aislamiento web evita cambios en Cargo. No se requiere ADR del runtime.

## Secuencia

1. Fundación Astro/Starlight, tokens, imágenes y layouts.
2. Landing y documentación con tutorial offline.
3. Blog y roadmap generado desde iniciativas.
4. Validadores, CI independiente y guía de despliegue.
5. Build, pruebas, revisión visual local y evidencia; commit/push autorizados.

## Validación y límites

Ver AN-056 y AN-057…060. Comprobar rutas, metadatos, fuentes, tutorial, temas y móvil.
No cambiar APIs Rust para corregir ejemplos. Si no hay toolchain local, ejecutar el
tutorial contra release explícita y comprobar el código actual en CI. Los resultados
no sustituyen aprobación humana ni certifican benchmarks. Publicación sigue pendiente.
