import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";

const envBin = process.env.WASMA_BIN || process.env.AGENT_NATIVE_BIN;
const executable = envBin
  ? resolve(envBin)
  : resolve(
      "../target/release/wasma" +
        (process.platform === "win32" ? ".exe" : ""),
    );
const tutorial = readFileSync(
  "src/content/docs/docs/first-component.md",
  "utf8",
);
const blocks = [...tutorial.matchAll(/```(\w+)([^\n]*)\n([\s\S]*?)```/g)];
const dir = mkdtempSync(join(tmpdir(), "wasma-docs-"));
try {
  for (const [, language, meta, body] of blocks) {
    const name = meta.match(/title="(identity\.(?:json|yaml))"/)?.[1];
    if (name) writeFileSync(join(dir, name), body);
    if (language !== "sh") continue;
    for (const command of body.trim().split("\n")) {
      assert.ok(
        command.startsWith("wasma "),
        "Only CLI tutorial commands are executable",
      );
      const args = command.slice("wasma ".length).trim().split(/\s+/);
      const result = spawnSync(executable, args, {
        cwd: dir,
        encoding: "utf8",
      });
      if (result.error) throw result.error;
      const expected = args.includes("missing") ? 1 : 0;
      assert.equal(
        result.status,
        expected,
        `${command}\n${result.stdout}\n${result.stderr}`,
      );
      if (args[0] === "run" && expected === 0)
        assert.match(result.stdout, /42/);
      console.log(`PASS (exit ${expected}): ${command}`);
    }
  }
  const result = spawnSync(
    executable,
    [
      "--format",
      "json",
      "run",
      "--component",
      "identity.wasm",
      "--function",
      "identity",
      "--fuel",
      "100000",
      "42",
    ],
    { cwd: dir, encoding: "utf8" },
  );
  assert.equal(result.status, 0);
  const json = JSON.parse(result.stdout);
  assert.equal(json.success, true);
  assert.equal(json.function_name, "identity");
  assert.deepEqual(json.results, [42]);
  console.log(
    "Tutorial executed directly from published Markdown; JSON output parsed.",
  );
} finally {
  rmSync(dir, { recursive: true, force: true });
}
