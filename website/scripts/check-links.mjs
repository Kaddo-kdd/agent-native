import { readdirSync, readFileSync, statSync, existsSync } from "node:fs";
import { resolve, relative, join, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { load } from "cheerio";

export function files(root) {
  return readdirSync(root).flatMap((name) => {
    const file = join(root, name);
    return statSync(file).isDirectory() ? files(file) : [file];
  });
}
export function checkLinks(root) {
  root = resolve(root);
  const errors = [];
  for (const file of files(root).filter((f) => f.endsWith(".html"))) {
    const route =
      "/" +
      relative(root, file)
        .split(sep)
        .join("/")
        .replace(/index\.html$/, "");
    const $ = load(readFileSync(file, "utf8"));
    $("a[href],img[src],script[src],link[href]").each((_, element) => {
      const value = $(element).attr("href") ?? $(element).attr("src");
      if (!value || /^(?:mailto:|tel:|data:|javascript:)/.test(value)) return;
      const url = new URL(value, `https://wasma.org${route}`);
      if (url.origin !== "https://wasma.org") return;
      // Canonicals are semantic URLs; 404.html has no routable /404/ counterpart.
      if ($(element).attr("rel") === "canonical") return;
      let target = resolve(root, "." + decodeURIComponent(url.pathname));
      if (!target.startsWith(root + sep) && target !== root) {
        errors.push(`${route}: path escape ${value}`);
        return;
      }
      if (existsSync(target) && statSync(target).isDirectory())
        target = join(target, "index.html");
      if (!existsSync(target)) {
        errors.push(`${route}: missing ${value}`);
        return;
      }
      if (url.hash && target.endsWith(".html")) {
        const anchor = decodeURIComponent(url.hash.slice(1));
        const doc = load(readFileSync(target, "utf8"));
        if (
          !doc("[id]")
            .toArray()
            .some((el) => doc(el).attr("id") === anchor)
        )
          errors.push(`${route}: missing fragment ${value}`);
      }
    });
  }
  return errors;
}
if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const errors = checkLinks("dist");
  if (errors.length) {
    console.error(errors.join("\n"));
    process.exitCode = 1;
  } else
    console.log("All generated internal routes, assets and fragments resolve.");
}
