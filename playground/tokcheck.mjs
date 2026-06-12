import { readFile, readdir } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import { Tiktoken } from "js-tiktoken/lite";
import o200k from "js-tiktoken/ranks/o200k_base";

const tk = new Tiktoken(o200k);
const dir = "../corpus";
const files = (await readdir(dir)).filter(f => /\.(curt|py|go|rs)$/.test(f));
let mismatches = 0, n = 0;
for (const f of files.sort()) {
  const src = await readFile(`${dir}/${f}`, "utf8");
  const js = tk.encode(src).length;
  const native = parseInt(execFileSync("../target/release/curt", ["tokens", "-"], { input: src }).toString().trim(), 10);
  n++;
  if (js !== native) { console.log(`MISMATCH ${f}: js=${js} native=${native}`); mismatches++; }
}
console.log(`\n${n} files checked, ${mismatches} mismatches`);
process.exit(mismatches ? 1 : 0);
