import { readFileSync } from "node:fs";
import { load } from "cheerio";
import { files } from "./check-links.mjs";
import { readRoadmap } from "../src/lib/roadmap.mjs";

readRoadmap();
const expected = [
  "docs",
  "docs/installation",
  "docs/first-component",
  "docs/concepts",
  "docs/cli",
  "docs/security",
  "docs/runtime",
  "docs/performance",
  "blog",
  "roadmap",
];
for (const route of expected) readFileSync(`dist/${route}/index.html`);
for (const file of files("dist").filter((f) => f.endsWith(".html"))) {
  const $ = load(readFileSync(file, "utf8"));
  for (const selector of [
    "title",
    'meta[name="description"]',
    'link[rel="canonical"]',
    'meta[property="og:title"]',
    'meta[property="og:description"]',
    'meta[property="og:image"]',
    "h1",
  ]) {
    if (!$(selector).length) throw new Error(`${file}: missing ${selector}`);
  }
  if ($("html").attr("lang") !== "es")
    throw new Error(`${file}: expected Spanish`);
  $("img").each((_, el) => {
    if ($(el).attr("alt") === undefined)
      throw new Error(`${file}: image without alt`);
  });
}
readFileSync("dist/sitemap-index.xml");
const tutorial = readFileSync(
  "src/content/docs/docs/first-component.md",
  "utf8",
);
if (
  !tutorial.includes('title="identity.json"') ||
  !tutorial.includes('title="identity.yaml"')
)
  throw new Error("Tutorial fixtures missing");
console.log(
  "Eight canonical roadmap blocks, required routes, SEO, language and tutorial fixtures verified.",
);
