// /gui-scrutiny — empirical, mechanical review of the playground in a real
// browser (chromium), light AND dark, with DOM assertions (not vibes). The
// load-bearing checks: every corpus program runs in-browser with the same
// stdout as the native interpreter, and the live token meter equals native
// `curt tokens`. Run with the static server already serving (default :8011):
//   python3 -m http.server -d . 8011 &
//   npx playwright test gui-scrutiny.spec.mjs
import { test, expect } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..");
const corpusDir = join(repo, "corpus");
const NATIVE = join(repo, "target", "release", "curt");
const BASE = process.env.PG_URL || "http://localhost:8011/";

function native(sub, src) {
  return execFileSync(NATIVE, [sub, "-"], { input: src }).toString();
}
const curtFiles = readdirSync(corpusDir).filter((f) => f.endsWith(".curt")).sort();

test.beforeEach(async ({ page }) => {
  const errors = [];
  page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
  page.on("pageerror", (e) => errors.push(String(e)));
  page.__errors = errors;
  await page.goto(BASE);
  await expect.poll(() => page.evaluate(() => document.body.dataset.ready)).toBe("true");
});

test("wasm loads with no console errors", async ({ page }) => {
  expect(await page.evaluate(() => document.body.dataset.ready)).toBe("true");
  await expect(page.getByTestId("status")).toContainText("ready");
  expect(page.__errors).toEqual([]);
});

test("every corpus program runs in-browser with native-identical stdout", async ({ page }) => {
  for (const f of curtFiles) {
    const src = readFileSync(join(corpusDir, f), "utf8");
    let expected;
    try { expected = native("run", src); } catch (e) { expected = (e.stdout || "").toString(); }
    const got = await page.evaluate(async (s) => {
      const { stdout } = await window.__curt.runCurt(s, { caps: ["--fs"] });
      return stdout;
    }, src);
    expect(got, `stdout mismatch for ${f}`).toBe(expected);
  }
});

test("live token meter equals native curt tokens (sampled programs)", async ({ page }) => {
  const editor = page.getByTestId("editor");
  for (const f of ["01_hello.curt", "08_pipeline.curt", "12_fold.curt"]) {
    const src = readFileSync(join(corpusDir, f), "utf8");
    const expected = native("tokens", src).trim();
    await editor.fill(src);
    await expect(page.getByTestId("token-count")).toHaveText(expected);
  }
});

test("curt-vs-Python table renders measured twin rows", async ({ page }) => {
  await page.getByTestId("tab-vs").click();
  await expect(page.getByTestId("panel-vs")).toBeVisible();
  const rows = page.locator('[data-testid="vs-row"]');
  expect(await rows.count()).toBeGreaterThan(5);
  await expect(page.getByTestId("vs-summary")).toContainText("%");
});

test("constrained-decode gallery renders samples + stats", async ({ page }) => {
  await page.getByTestId("tab-gallery").click();
  await expect(page.getByTestId("panel-gallery")).toBeVisible();
  expect(await page.getByTestId("gallery-sample").count()).toBeGreaterThan(0);
  await expect(page.getByTestId("gallery-stats")).toContainText("0");
});

for (const theme of ["light", "dark"]) {
  test(`theme ${theme}: mechanical + screenshot`, async ({ page }) => {
    // Toggle to the target theme deterministically.
    const current = await page.evaluate(() => document.documentElement.getAttribute("data-theme"));
    if (current !== theme) await page.getByTestId("theme-toggle").click();
    await expect.poll(() => page.evaluate(() => document.documentElement.getAttribute("data-theme"))).toBe(theme);
    // Key surfaces present and the run path still works under this theme.
    await page.getByTestId("tab-run").click();
    await expect(page.getByTestId("editor")).toBeVisible();
    await expect(page.getByTestId("run-btn")).toBeVisible();
    await page.getByTestId("run-btn").click();
    await expect(page.getByTestId("status")).toContainText("ok");
    mkdirSync(join(here, "screenshots"), { recursive: true });
    await page.screenshot({ path: join(here, "screenshots", `playground-${theme}.png`), fullPage: true });
  });
}
