---
title: curt playground
emoji: ⚡
colorFrom: orange
colorTo: gray
sdk: static
app_file: index.html
pinned: false
license: mit
---

# curt — browser playground

A backend-free playground for [curt](https://github.com/AlrikOlson/curt), a
machine-first programming language where the tokenizer is the ISA. Everything
runs in your browser:

- **Run** — the `wasm32-wasip1` build of the reference interpreter (`curt.wasm`)
  executes your program client-side via a pure-JS WASI shim. No server.
- **Live token meter** — `o200k_base`, the same tokenizer the language is
  optimized against. Verified token-for-token against the native
  `curt tokens` across the whole corpus (see `tokcheck.mjs`): 0 mismatches.
- **curt vs Python** — the corpus twins, counted side by side.
- **Constrained decode** — the grammar-mask reliability story, sampled.

## Develop

```sh
npm install
npm run build        # → dist/app.bundle.js, curt.wasm, src/generated/data.js
python3 -m http.server -d . 8000   # then open http://localhost:8000
```

The build regenerates everything from the repo's `corpus/` and
`tools/grammar/results.json`, then bundles with esbuild. The wasm artifact comes
from `cargo build --bin curt --release --target wasm32-wasip1 --no-default-features`
(built in CI; the token meter is JS-side precisely because that build omits the
`tokens` feature).

## Verify

```sh
node tokcheck.mjs    # asserts JS o200k == native curt tokens over the corpus
```

`gui-scrutiny.spec.mjs` drives the page in a real browser (Playwright, light and
dark) with mechanical DOM assertions.

## Deploy to a Hugging Face Space

This directory is a ready-to-serve static Space (`sdk: static`). Push it to a
Space repo (`git push` to `hf.co/spaces/<user>/curt-playground`) — credentialed,
so it's a user action.
