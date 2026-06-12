// Static build for curtlang.com — a multi-page brand/info hub deployed to
// GitHub Pages. Zero runtime dependencies: pages are assembled from a shared
// layout at build time (no client framework), and the live playground (its
// own self-contained static bundle) is copied in under /play/.
//
//   node site/build.mjs        → writes site/dist/
import {
  readFileSync, writeFileSync, mkdirSync, readdirSync, copyFileSync, rmSync, statSync, existsSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { dirname, join, relative } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..");
const dist = join(here, "dist");
const SITE = "https://curtlang.com";

// Content-hash the styles/script so each change busts the CDN + browser cache.
// (A stable /assets/brand.css URL silently served a stale stylesheet after a
// deploy — versioned URLs make every asset change fetch fresh.)
const hash8 = (p) => createHash("sha256").update(readFileSync(p)).digest("hex").slice(0, 8);
const CSS_V = hash8(join(here, "assets", "brand.css"));
const JS_V = hash8(join(here, "assets", "site.js"));

// ---- nav + layout ----------------------------------------------------------
const NAV = [
  { href: "/", label: "Home", slug: "home" },
  { href: "/language", label: "Language", slug: "language" },
  { href: "/benchmarks", label: "Benchmarks", slug: "benchmarks" },
  { href: "/play/", label: "Playground", slug: "play" },
];
const EXT = [
  { href: "https://github.com/AlrikOlson/curt", label: "GitHub" },
  { href: "https://huggingface.co/datasets/therikkening/curt-benchmarks", label: "Dataset" },
];

function navHtml(active) {
  const inner = NAV.map((n) =>
    `<a href="${n.href}"${n.slug === active ? ' aria-current="page"' : ""}>${n.label}</a>`).join("");
  const ext = EXT.map((n) => `<a class="ext" href="${n.href}">${n.label}</a>`).join("");
  return `<header class="nav" data-testid="nav">
  <a class="brand" href="/" aria-label="curt home"><span class="brand-mark" aria-hidden="true"></span><span class="brand-word">curt</span></a>
  <button class="nav-toggle" id="nav-toggle" aria-label="Menu" aria-expanded="false">≡</button>
  <nav class="nav-links" id="nav-links">${inner}<span class="nav-sep"></span>${ext}
    <button class="theme-toggle" id="theme-toggle" data-testid="theme-toggle" aria-label="Toggle theme"><span class="dot" aria-hidden="true"></span><span class="theme-label">Dark</span></button>
  </nav>
</header>`;
}

const FOOTER = `<footer class="footer" data-testid="footer">
  <div class="foot-grid">
    <div>
      <div class="brand small"><span class="brand-mark" aria-hidden="true"></span><span class="brand-word">curt</span></div>
      <p class="dim">A machine-first programming language for AI agents. The tokenizer is the ISA.</p>
      <p class="dim tiny">Measured, never estimated — every number on this site reproduces from a committed script.</p>
    </div>
    <div>
      <h4>Project</h4>
      <a href="https://github.com/AlrikOlson/curt">GitHub</a>
      <a href="https://github.com/AlrikOlson/curt/blob/main/SPEC.md">Specification</a>
      <a href="https://github.com/AlrikOlson/curt/blob/main/DESIGN.md">Design notes</a>
      <a href="https://github.com/AlrikOlson/curt/blob/main/ROADMAP.md">Roadmap</a>
    </div>
    <div>
      <h4>Explore</h4>
      <a href="/language">Language tour</a>
      <a href="/benchmarks">Benchmarks</a>
      <a href="/play/">Browser playground</a>
      <a href="https://huggingface.co/datasets/therikkening/curt-benchmarks">Benchmark dataset</a>
    </div>
  </div>
  <div class="foot-base dim tiny">
    <span>MIT / Apache-2.0</span><span class="sep">·</span>
    <span>Status: v0.3.1 — the full toolchain runs.</span>
  </div>
</footer>`;

function layout({ title, desc, active, main, ogPath }) {
  return `<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>${title}</title>
<meta name="description" content="${desc}" />
<meta name="theme-color" content="#0b0d10" />
<meta property="og:title" content="${title}" />
<meta property="og:description" content="${desc}" />
<meta property="og:type" content="website" />
<meta property="og:url" content="${SITE}${ogPath}" />
<meta name="twitter:card" content="summary_large_image" />
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='6' fill='%23b6f24a'/%3E%3Ctext x='16' y='23' font-family='monospace' font-size='20' font-weight='700' text-anchor='middle' fill='%230b0d10'%3Ec%3C/text%3E%3C/svg%3E" />
<link rel="stylesheet" href="/assets/brand.css?v=${CSS_V}" />
<script>(function(){try{var t=localStorage.getItem('curt-theme');if(!t)t=matchMedia('(prefers-color-scheme: light)').matches?'light':'dark';document.documentElement.setAttribute('data-theme',t);}catch(e){}})();</script>
</head>
<body>
${navHtml(active)}
<main>
${main}
</main>
${FOOTER}
<script src="/assets/site.js?v=${JS_V}" defer></script>
</body>
</html>`;
}

// ---- pages manifest --------------------------------------------------------
const PAGES = [
  { file: "index.html",      out: "index.html",      slug: "home",       og: "/",                title: "curt — a machine-first programming language for AI agents", desc: "curt is a statically-typed, compiled language where output-token cost is the prime design directive. The tokenizer is the ISA." },
  { file: "language.html",   out: "language.html",   slug: "language",   og: "/language",   title: "The curt language — a tour", desc: "Equations and juxtaposition, untagged unions with full inference, a capability model, and machine-readable diagnostics. A guided tour of curt." },
  { file: "benchmarks.html", out: "benchmarks.html", slug: "benchmarks", og: "/benchmarks", title: "curt benchmarks — measured, never estimated", desc: "Token cost vs Python/Go/Rust, the curt-vs-Zerolang head-to-head, the diagnostics tournament, and grammar-masked generation — every number reproduces." },
];

// ---- build -----------------------------------------------------------------
function rmrf(p) { if (existsSync(p)) rmSync(p, { recursive: true, force: true }); }
function copyDir(src, dst) {
  mkdirSync(dst, { recursive: true });
  for (const e of readdirSync(src)) {
    const s = join(src, e), d = join(dst, e);
    if (statSync(s).isDirectory()) copyDir(s, d);
    else copyFileSync(s, d);
  }
}

rmrf(dist);
mkdirSync(dist, { recursive: true });

// pages
for (const p of PAGES) {
  const main = readFileSync(join(here, "pages", p.file), "utf8");
  writeFileSync(join(dist, p.out), layout({ title: p.title, desc: p.desc, active: p.slug, main, ogPath: p.og }));
}

// assets
copyDir(join(here, "assets"), join(dist, "assets"));

// the live playground (its committed static bundle) → /play/
const pg = join(repo, "playground");
mkdirSync(join(dist, "play", "dist"), { recursive: true });
for (const [s, d] of [
  ["styles.css", "styles.css"],
  ["curt.wasm", "curt.wasm"],
  ["dist/app.bundle.js", "dist/app.bundle.js"],
]) copyFileSync(join(pg, s), join(dist, "play", d));
// version the playground's own asset refs (same cache-bust discipline)
const pgCss = hash8(join(pg, "styles.css"));
const pgJs = hash8(join(pg, "dist", "app.bundle.js"));
const pgHtml = readFileSync(join(pg, "index.html"), "utf8")
  .replace('href="./styles.css"', `href="./styles.css?v=${pgCss}"`)
  .replace('src="./dist/app.bundle.js"', `src="./dist/app.bundle.js?v=${pgJs}"`);
writeFileSync(join(dist, "play", "index.html"), pgHtml);

console.log(`built site → ${relative(repo, dist)} (${PAGES.length} pages + /play/)`);
