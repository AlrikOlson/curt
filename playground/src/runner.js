// Browser-side curt runner: loads the wasm32-wasip1 build and executes a
// program entirely client-side via a pure-JS WASI shim. The program is fed
// on stdin (`curt run -`) and stdout/stderr are captured by reading back the
// in-memory WASI files. No backend, no network.
import { WASI, OpenFile, File } from "@bjorn3/browser_wasi_shim";

const enc = new TextEncoder();
const dec = new TextDecoder();

let _module = null;

// Compile curt.wasm once; instantiate fresh per run (wasi.start consumes the
// instance). compile(arrayBuffer) — not compileStreaming — so a static host
// that serves curt.wasm without the application/wasm mime type still works.
export async function loadCurt(url = "./curt.wasm") {
  if (!_module) {
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`fetch ${url}: ${resp.status}`);
    _module = await WebAssembly.compile(await resp.arrayBuffer());
  }
  return _module;
}

// Run a curt program. caps is an array of capability flags, e.g. ["--fs"].
// Returns { code, stdout, stderr }. The wasm is built --no-default-features,
// so net ops yield rescuable err values and `tokens` is unavailable (the
// token meter is JS-side; see tokens.js).
export async function runCurt(program, { caps = [] } = {}) {
  const mod = await loadCurt();
  const outFile = new File([]);
  const errFile = new File([]);
  const fds = [
    new OpenFile(new File(enc.encode(program))), // fd 0 stdin
    new OpenFile(outFile), // fd 1 stdout
    new OpenFile(errFile), // fd 2 stderr
  ];
  const wasi = new WASI(["curt", "run", ...caps, "-"], [], fds, { debug: false });
  const instance = await WebAssembly.instantiate(mod, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  let code = 0;
  try {
    code = wasi.start(instance);
  } catch (e) {
    if (e && e.constructor && e.constructor.name === "WASIProcExit") code = e.code;
    else throw e;
  }
  return { code, stdout: dec.decode(outFile.data), stderr: dec.decode(errFile.data) };
}
