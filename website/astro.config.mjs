import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import mdx from "@astrojs/mdx";

export default defineConfig({
  site: "https://wasma.org",
  trailingSlash: "always",
  output: "static",
  integrations: [
    starlight({
      title: "Wasma",
      description:
        "De la especificación a componentes WebAssembly verificables.",
      defaultLocale: "root",
      locales: { root: { label: "Español", lang: "es" } },
      disable404Route: true,
      favicon: "/brand/favicon.svg",
      editLink: {
        baseUrl: "https://github.com/Kaddo-kdd/agent-native/edit/main/website/",
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/Kaddo-kdd/agent-native",
        },
      ],
      customCss: ["./src/styles/docs.css"],
      components: { SiteTitle: "./src/components/DocsTitle.astro" },
      head: [
        {
          tag: "meta",
          attrs: {
            property: "og:image",
            content: "https://wasma.org/brand/social.png",
          },
        },
        ...(process.env.CONTEXT === "deploy-preview"
          ? [
              {
                tag: "meta",
                attrs: { name: "robots", content: "noindex,nofollow" },
              },
            ]
          : []),
      ],
      sidebar: [
        {
          label: "Empezar",
          items: [
            { label: "Qué es Wasma", slug: "docs" },
            { label: "Instalación", slug: "docs/installation" },
            { label: "Primer componente", slug: "docs/first-component" },
          ],
        },
        {
          label: "Entender y construir",
          items: [
            { label: "Conceptos", slug: "docs/concepts" },
            { label: "Referencia CLI", slug: "docs/cli" },
            { label: "Runtime", slug: "docs/runtime" },
            { label: "Seguridad", slug: "docs/security" },
            { label: "Rendimiento", slug: "docs/performance" },
          ],
        },
        {
          label: "Proyecto",
          items: [
            { label: "Inicio", link: "/" },
            { label: "Roadmap", link: "/roadmap/" },
            { label: "Blog", link: "/blog/" },
          ],
        },
      ],
    }),
    mdx(),
  ],
});
