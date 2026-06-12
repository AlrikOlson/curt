// Minimal JS embedding of curt via node:wasi (WASI preview1).
// Run from the repo root after `cargo build --release --target
// wasm32-wasip1 --no-default-features`:
//   node examples/embed/run.mjs
import { readFile, writeFile, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { WASI } from "node:wasi";
import { spawnSync } from "node:child_process";

const program = 'xs = [1, 2, 3, 4]\nprint (xs | keep (x -> x > 1) | sum)\n';

const dir = await mkdtemp(join(tmpdir(), "curt-embed-"));
await writeFile(join(dir, "main.curt"), program);

// node:wasi has no in-process stdout capture knob, so the capture-friendly
// path is a re-exec with stdio pipes — same module, same preopen model.
const out = spawnSync(process.execPath, ["--input-type=module", "-e", `
  import { readFile } from "node:fs/promises";
  import { WASI } from "node:wasi";
  const wasi = new WASI({ version: "preview1",
    args: ["curt", "run", "main.curt"], preopens: { ".": ${JSON.stringify(dir)} } });
  const wasm = await WebAssembly.compile(
    await readFile("target/wasm32-wasip1/release/curt.wasm"));
  const inst = await WebAssembly.instantiate(wasm, wasi.getImportObject());
  process.exitCode = wasi.start(inst);
`], { encoding: "utf8" });

await rm(dir, { recursive: true, force: true });
if (out.status !== 0) {
  console.error(out.stderr);
  process.exit(1);
}
const stdout = out.stdout;
console.log(`captured: ${JSON.stringify(stdout)}`);
if (stdout !== "9\n") {
  console.error("expected '9\\n'");
  process.exit(1);
}
console.log("JS embedding OK");
