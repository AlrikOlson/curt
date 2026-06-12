#!/usr/bin/env bash
# wasm gate (chunk:wasm-embed): the wasip1 build must run the corpus under
# wasmtime with golden-identical stdout, and net ops must yield rescuable
# err values (language semantics unchanged without the net feature).
#
# 20_server.curt is excluded from the golden sweep (it binds a TCP port,
# which the no-net wasm build correctly cannot) — its behavior is covered
# by the explicit net-err probe below instead.

set -euo pipefail
cd "$(dirname "$0")/.."

WASM=target/wasm32-wasip1/release/curt.wasm

if ! command -v wasmtime >/dev/null; then
  echo "wasmtime not found — install it (brew install wasmtime / wasmtime.dev/install.sh)" >&2
  exit 1
fi

echo "building wasm32-wasip1 (no default features)"
# --bin only: the cdylib lib target (host-ffi) would collide on curt.wasm
cargo build --bin curt --release --target wasm32-wasip1 --no-default-features

# the guest sees ONLY the preopened fixtures dir (no .. escape), so the
# program file is copied inside it and run with a guest-relative path
fails=0
for f in corpus/*.curt; do
  name=$(basename "$f")
  [ "$name" = "20_server.curt" ] && continue
  cp "$f" tests/fixtures/__smoke.curt
  native=$(cd tests/fixtures && ../../target/release/curt run --fs __smoke.curt 2>/dev/null || true)
  wasm=$(wasmtime run --dir tests/fixtures::. "$WASM" run --fs __smoke.curt 2>/dev/null || true)
  rm -f tests/fixtures/__smoke.curt
  if [ "$native" != "$wasm" ]; then
    echo "  MISMATCH $name" >&2
    fails=$((fails + 1))
  else
    echo "  PASS $name"
  fi
done

# net ops yield a rescuable err value in the no-net build
out=$(printf 'print (net.listen 8080 ? "rescued")\n' | wasmtime run "$WASM" run --net - 2>&1)
if [ "$out" != "rescued" ]; then
  echo "  net-err probe FAILED: got '$out'" >&2
  fails=$((fails + 1))
else
  echo "  PASS net-err probe (rescuable err without the net feature)"
fi

[ "$fails" -eq 0 ] || { echo "wasm smoke: $fails failure(s)" >&2; exit 1; }
echo "wasm smoke: corpus golden-identical under wasmtime"
