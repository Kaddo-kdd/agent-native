import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { checkLinks } from "./check-links.mjs";
import { readRoadmap } from "../src/lib/roadmap.mjs";

test("link check rejects missing files and fragments, accepts real fragments", () => {
  const root = mkdtempSync(join(tmpdir(), "wasma-links-"));
  try {
    writeFileSync(
      join(root, "index.html"),
      '<a href="/missing/">broken</a><a href="/#absent">broken</a><a href="/#present">ok</a><h1 id="present">Title</h1>',
    );
    const errors = checkLinks(root);
    assert.equal(errors.length, 2);
    assert.ok(errors.some((e) => e.includes("missing fragment")));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
test("roadmap reflects canonical state and rejects missing or unknown states", () => {
  const root = mkdtempSync(join(tmpdir(), "wasma-roadmap-"));
  const write = (n, status) =>
    writeFileSync(
      join(root, `INI-00${n}-test.md`),
      `---\nid: INI-00${n}\ntitle: RM-0${n} — Test\nroadmap_block: RM-0${n}\nstatus: ${status}\n---\n`,
    );
  try {
    for (let n = 1; n <= 8; n++) write(n, "completed");
    assert.equal(readRoadmap(root)[7].status, "completed");
    write(8, "in-progress");
    assert.equal(readRoadmap(root)[7].label, "En curso");
    write(8, "unrecognized");
    assert.throws(() => readRoadmap(root), /Unknown status/);
    rmSync(join(root, "INI-008-test.md"));
    assert.throws(() => readRoadmap(root), /eight/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
