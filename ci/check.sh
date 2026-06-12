#!/usr/bin/env bash
# curt CI — the single source of gate truth.
#
# Local dev and .github/workflows/ci.yml both run THIS script, so a gate
# cannot be weakened in one place without weakening it everywhere.
# Every gate here is load-bearing; never mask an exit code.

set -euo pipefail
cd "$(dirname "$0")/.."

say() { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }

say "python venv (tiktoken, parsimonious, lark)"
PY="${CURT_CI_PYTHON:-python3}"
VENV=".ci-venv"
if [ ! -x "$VENV/bin/python" ]; then
  "$PY" -m venv "$VENV"
  "$VENV/bin/pip" -q install --upgrade pip
fi
"$VENV/bin/pip" -q install tiktoken parsimonious lark

say "cargo test"
cargo test

say "cargo clippy -D warnings"
cargo clippy --all-targets -- -D warnings

say "release build (oracle for the grammar gates)"
cargo build --release

say "PEG grammar gate: corpus (all .curt files)"
"$VENV/bin/python" tools/tokens/validate.py

say "Lark grammar gate: corpus (all .curt files) + negative agreement"
"$VENV/bin/python" tools/grammar/validate.py

say "GBNF determinism: regeneration is byte-identical"
cp tools/grammar/curt.gbnf /tmp/curt-ci-gbnf.$$
"$VENV/bin/python" tools/grammar/lark2gbnf.py > /dev/null
diff /tmp/curt-ci-gbnf.$$ tools/grammar/curt.gbnf
rm -f /tmp/curt-ci-gbnf.$$

say "cheat sheet: <=2500 o200k tokens + docs/llms.txt not stale"
"$VENV/bin/python" - <<'EOF'
import tiktoken, pathlib, sys
n = len(tiktoken.get_encoding("o200k_base").encode(pathlib.Path("CHEATSHEET.md").read_text()))
print(f"CHEATSHEET.md: {n} o200k tokens (ceiling 2500)")
sys.exit(0 if n <= 2500 else 1)
EOF
"$VENV/bin/python" tools/cheatsheet/emit_llms.py --check

say "wasm32-wasip1 gate: build + corpus smoke under wasmtime"
rustup target add wasm32-wasip1 --toolchain "$(rustup show active-toolchain | cut -d' ' -f1)" >/dev/null 2>&1 || rustup target add wasm32-wasip1 >/dev/null
./ci/wasm_smoke.sh

say "token cost table (regression evidence in the log)"
"$VENV/bin/python" tools/tokens/count.py

say "ALL GATES GREEN"
