import { readdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { parse } from "yaml";

// Astro bundles server modules into dist before prerendering; import.meta.url moves.
// All package scripts run with website/ as cwd, including the hosting build.
export const initiativeRoot = resolve(
  process.cwd(),
  "../knowledge/delivery/initiatives",
);
export const statusLabels = {
  completed: "Completado",
  "in-progress": "En curso",
  draft: "Propuesto",
  candidate: "Planeado",
  ready: "Preparado",
  blocked: "Bloqueado",
};
export function readRoadmap(root = initiativeRoot) {
  const rows = readdirSync(root)
    .filter((name) => /^INI-00[1-8]-.*\.md$/.test(name))
    .map((name) => {
      const text = readFileSync(join(root, name), "utf8");
      const front = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
      if (!front) throw new Error(`Missing frontmatter: ${name}`);
      // Only the four public scalar fields are part of this projection. Historical
      // candidate notes contain free-form colons and are neither parsed nor exposed.
      const fields = front[1]
        .split(/\r?\n/)
        .filter((line) => /^(id|title|status|roadmap_block):/.test(line));
      const data = parse(fields.join("\n"));
      if (typeof data.id !== "string" || typeof data.title !== "string")
        throw new Error(`Missing public fields: ${name}`);
      if (!Object.hasOwn(statusLabels, data.status))
        throw new Error(`Unknown status: ${name}: ${data.status}`);
      if (!/^RM-0[1-8]$/.test(data.roadmap_block))
        throw new Error(`Invalid roadmap block: ${name}`);
      return {
        id: data.id,
        block: data.roadmap_block,
        title: data.title.replace(/^RM-\d+\s*[—-]\s*/, ""),
        status: data.status,
        label: statusLabels[data.status],
        source: `https://github.com/Kaddo-kdd/wasma/blob/main/knowledge/delivery/initiatives/${name}`,
      };
    })
    .sort((a, b) => a.block.localeCompare(b.block));
  if (rows.length !== 8 || new Set(rows.map((row) => row.block)).size !== 8)
    throw new Error("Expected eight unique roadmap blocks");
  return rows;
}
