// curt playground — wiring. Everything runs client-side.
import { runCurt, loadCurt } from "./runner.js";
import { countTokens } from "./tokens.js";
import { EXAMPLES, GALLERY } from "./generated/data.js";

const $ = (sel) => document.querySelector(sel);

// ---- theme -----------------------------------------------------------------
function initTheme() {
  const toggle = $("#theme-toggle");
  const prefersDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
  setTheme(prefersDark ? "dark" : "light");
  toggle.addEventListener("click", () => {
    const next = document.documentElement.getAttribute("data-theme") === "dark" ? "light" : "dark";
    setTheme(next);
  });
}
function setTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
  const label = $("#theme-toggle .theme-toggle-label");
  if (label) label.textContent = theme === "dark" ? "Light" : "Dark";
}

// ---- tabs ------------------------------------------------------------------
function initTabs() {
  const tabs = document.querySelectorAll(".tab");
  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      tabs.forEach((t) => t.setAttribute("aria-selected", String(t === tab)));
      const which = tab.getAttribute("data-tab");
      document.querySelectorAll(".panel").forEach((p) => {
        p.hidden = p.getAttribute("data-panel") !== which;
      });
    });
  });
}

// ---- token meter -----------------------------------------------------------
function updateMeter() {
  $("#token-count").textContent = String(countTokens($("#editor").value));
}

// ---- run -------------------------------------------------------------------
async function run() {
  const btn = $("#run-btn");
  btn.disabled = true;
  $("#status").textContent = "running…";
  const stdoutEl = $("#stdout");
  const stderrEl = $("#stderr");
  const stderrLabel = $("#stderr-label");
  try {
    const { code, stdout, stderr } = await runCurt($("#editor").value, { caps: ["--fs"] });
    stdoutEl.textContent = stdout;
    stdoutEl.dataset.exit = String(code);
    const hasErr = stderr.trim().length > 0;
    stderrEl.hidden = !hasErr;
    stderrLabel.hidden = !hasErr;
    stderrEl.textContent = stderr;
    $("#status").textContent = code === 0 ? "ok (exit 0)" : `exit ${code}`;
  } catch (e) {
    stdoutEl.textContent = "";
    stderrEl.hidden = false;
    stderrLabel.hidden = false;
    stderrEl.textContent = String(e);
    $("#status").textContent = "error";
  } finally {
    btn.disabled = false;
  }
}

// ---- examples --------------------------------------------------------------
function initExamples() {
  const select = $("#example-select");
  EXAMPLES.forEach((ex, i) => {
    const opt = document.createElement("option");
    opt.value = String(i);
    opt.textContent = ex.name;
    select.appendChild(opt);
  });
  select.addEventListener("change", () => loadExample(Number(select.value)));
  loadExample(0);
}
function loadExample(i) {
  const ex = EXAMPLES[i];
  if (!ex) return;
  $("#editor").value = ex.curt;
  updateMeter();
  $("#stdout").textContent = "";
  $("#stderr").hidden = true;
  $("#stderr-label").hidden = true;
}

// ---- curt vs Python --------------------------------------------------------
function initVs() {
  const body = $("#vs-body");
  const rows = EXAMPLES.filter((ex) => ex.py != null);
  let sumCurt = 0, sumPy = 0;
  rows.forEach((ex) => {
    const c = countTokens(ex.curt);
    const p = countTokens(ex.py);
    sumCurt += c; sumPy += p;
    const savings = p > 0 ? Math.round((1 - c / p) * 100) : 0;
    const tr = document.createElement("tr");
    tr.dataset.testid = "vs-row";
    tr.innerHTML =
      `<td class="ex-name">${escapeHtml(ex.name)}</td>` +
      `<td class="num">${c}</td>` +
      `<td class="num">${p}</td>` +
      `<td class="num ${savings >= 0 ? "good" : "bad"}">${savings >= 0 ? "−" : "+"}${Math.abs(savings)}%</td>`;
    body.appendChild(tr);
  });
  const totalSavings = sumPy > 0 ? Math.round((1 - sumCurt / sumPy) * 100) : 0;
  $("#vs-summary").innerHTML =
    `<span class="big">${totalSavings >= 0 ? "−" : "+"}${Math.abs(totalSavings)}%</span>` +
    `<span class="big-label">total o200k tokens across ${rows.length} twins · curt ${sumCurt} vs Python ${sumPy}</span>`;
}

// ---- gallery ---------------------------------------------------------------
function initGallery() {
  $("#gallery-blurb").textContent = GALLERY.blurb;
  const stats = $("#gallery-stats");
  GALLERY.stats.forEach((s) => {
    const div = document.createElement("div");
    div.className = "stat";
    div.innerHTML = `<span class="stat-num">${escapeHtml(s.value)}</span><span class="stat-label">${escapeHtml(s.label)}</span>`;
    stats.appendChild(div);
  });
  const grid = $("#gallery-grid");
  GALLERY.samples.forEach((s) => {
    const card = document.createElement("div");
    card.className = "sample " + (s.parses ? "pass" : "fail");
    card.dataset.testid = "gallery-sample";
    card.dataset.parses = String(s.parses);
    card.innerHTML =
      `<div class="sample-head"><span class="badge">${s.parses ? "parses ✓" : "prefix-only"}</span></div>` +
      `<pre class="sample-code">${escapeHtml(s.out)}</pre>`;
    grid.appendChild(card);
  });
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}

// ---- boot ------------------------------------------------------------------
async function boot() {
  initTheme();
  initTabs();
  initExamples();
  initVs();
  initGallery();
  $("#editor").addEventListener("input", updateMeter);
  $("#run-btn").addEventListener("click", run);
  // Test affordance for gui-scrutiny: run programs / count tokens off-DOM.
  window.__curt = { runCurt, countTokens };
  try {
    await loadCurt();
    $("#status").textContent = "ready — curt.wasm loaded";
    document.body.dataset.ready = "true";
  } catch (e) {
    $("#status").textContent = "failed to load curt.wasm: " + e;
  }
}

boot();
