import { readdirSync, readFileSync, writeFileSync } from "node:fs";
// A viewport around the original icon, using Astro's already optimized asset.
// No vector tracing, recoloring or raster modification is performed here.
const name = readdirSync("dist/_astro").find((name) =>
  /^wasma-lockup\..*\.webp$/.test(name),
);
if (!name) throw new Error("Optimized brand image missing");
const base64 = readFileSync(`dist/_astro/${name}`).toString("base64");
writeFileSync(
  "dist/brand/favicon.svg",
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="680 705 215 235"><image width="1448" height="1086" href="data:image/webp;base64,${base64}"/></svg>`,
);
console.log(
  "Favicon packages the original icon from the optimized brand asset.",
);
