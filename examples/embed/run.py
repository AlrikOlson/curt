#!/usr/bin/env python3
"""Minimal Python embedding of curt via wasmtime-py (WASI preview1).

Run from the repo root after building the wasm artifact:
  pip install wasmtime && python examples/embed/run.py
"""

import sys
import tempfile
from pathlib import Path

from wasmtime import Config, Engine, Linker, Module, Store, WasiConfig

PROGRAM = 'xs = [1, 2, 3, 4]\nprint (xs | keep (x -> x > 1) | sum)\n'
WASM = Path("target/wasm32-wasip1/release/curt.wasm")


def main() -> int:
    with tempfile.TemporaryDirectory() as td:
        (Path(td) / "main.curt").write_text(PROGRAM)
        out_path = Path(td) / "stdout.txt"

        wasi = WasiConfig()
        wasi.argv = ["curt", "run", "main.curt"]
        wasi.preopen_dir(td, ".")
        wasi.stdout_file = str(out_path)

        engine = Engine(Config())
        store = Store(engine)
        store.set_wasi(wasi)
        linker = Linker(engine)
        linker.define_wasi()
        module = Module.from_file(engine, str(WASM))
        instance = linker.instantiate(store, module)
        instance.exports(store)["_start"](store)

        stdout = out_path.read_text()

    print(f"captured: {stdout!r}")
    if stdout != "9\n":
        print("expected '9\\n'", file=sys.stderr)
        return 1
    print("Python embedding OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
