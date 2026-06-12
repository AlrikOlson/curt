# curtlang.com — the brand/info site

A multi-page static hub for [curt](https://github.com/AlrikOlson/curt),
deployed to **Cloudflare Pages** (project `curtlang`, live at
https://curtlang.pages.dev). Structure is documented in
[`docs/UX-BLUEPRINT.md`](../docs/UX-BLUEPRINT.md) (spatial annotated specimen).

## Build

Zero runtime dependencies — pages are assembled from a shared layout at build
time, and the committed playground bundle is copied in under `/play/`:

```sh
node site/build.mjs        # → site/dist/  (gitignored build output)
python3 -m http.server -d site/dist 8021   # preview at http://localhost:8021
```

Pages: `pages/index.html` (home — rotating annotated specimens),
`pages/language.html` (annotated tour), `pages/benchmarks.html` (proof-ledger).
Every token count is a committed measurement (`curt tokens`, o200k) with its
reproduce path shown beside it.

## Deploy

Manual (what `launch` does):

```sh
node site/build.mjs
wrangler pages deploy site/dist --project-name=curtlang --branch=main
```

For push-to-deploy, connect this repo to the `curtlang` Pages project in the
Cloudflare dashboard (Workers & Pages → curtlang → Settings → Builds &
deployments → Connect to Git; build command `node site/build.mjs`, output
`site/dist`). No GitHub Actions secrets needed — Cloudflare builds on push.

## Custom domain (curtlang.com)

Attach in the dashboard: Workers & Pages → `curtlang` → Custom domains →
*Set up a custom domain* → `curtlang.com`. Since the zone is in the same
Cloudflare account, the DNS record is created automatically.

## Verify

`/gui-scrutiny` runs empirically against a local server (Playwright, light +
dark): home specimen rotation + code↔annotation linking, `/play/` wasm load,
the ledger and comparison tables, with full-page screenshots in both themes.
