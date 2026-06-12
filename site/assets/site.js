// curtlang.com — progressive enhancement: theme toggle, mobile nav, and the
// rotating annotated specimen on the home page. No dependencies.
(function () {
  // ---- theme ----
  var root = document.documentElement;
  var tt = document.getElementById("theme-toggle");
  function setTheme(t) {
    root.setAttribute("data-theme", t);
    try { localStorage.setItem("curt-theme", t); } catch (e) {}
    var lbl = tt && tt.querySelector(".theme-label");
    if (lbl) lbl.textContent = t === "dark" ? "Light" : "Dark";
  }
  if (tt) {
    var cur = root.getAttribute("data-theme") || "dark";
    var lbl = tt.querySelector(".theme-label");
    if (lbl) lbl.textContent = cur === "dark" ? "Light" : "Dark";
    tt.addEventListener("click", function () {
      setTheme(root.getAttribute("data-theme") === "dark" ? "light" : "dark");
    });
  }

  // ---- mobile nav ----
  var navToggle = document.getElementById("nav-toggle");
  var navLinks = document.getElementById("nav-links");
  if (navToggle && navLinks) {
    navToggle.addEventListener("click", function () {
      var open = navLinks.classList.toggle("open");
      navToggle.setAttribute("aria-expanded", String(open));
    });
  }

  // ---- rotating annotated specimen ----
  var code = document.getElementById("spec-code");
  if (!code) return;

  // Real corpus programs; token counts are o200k_base, measured via
  // `curt tokens` (and tiktoken o200k — verified identical). Sources:
  // corpus/20_server.curt, 10_group.curt, 07_errors.curt; the server's
  // py/go/rust counts are DESIGN.md's three-program measurement.
  var SPECIMENS = [
    {
      title: "tcp uppercase echo",
      code:
        'handle c = <a1>for</a1> ln <a1>in</a1> c.<a2>lines</a2> {\n' +
        '  c.<a2>write</a2> (ln.<a2>upper</a2> + <s>"\\n"</s>) }\n' +
        '<a1>for</a1> c <a1>in</a1> <a3>net.listen</a3> 8080 { <a4>go</a4> handle c }',
      annot: [
        { n: 1, meaning: "This whole line defines a function — no keywords like <code>def</code> or <code>return</code>, no parentheses. Less to type means fewer tokens.", cost: "the whole header · ~3 tokens" },
        { n: 2, meaning: "<code>c.lines</code> and <code>ln.upper</code> read almost like English — common actions are single short words, so they cost almost nothing.", cost: "1–2 tokens each" },
        { n: 3, meaning: "Reaching the network needs permission. By default a curt program can't — so it can't quietly misbehave.", cost: "net.listen · 3 tokens" },
        { n: 4, meaning: "<code>go</code> runs something in the background — one tiny word does what's usually a page of setup.", cost: "go · 1 token" },
      ],
      compare: [{ lang: "curt", n: 32, win: true }, { lang: "python", n: 55 }, { lang: "go", n: 94 }, { lang: "rust", n: 123 }],
      repro: { label: "DESIGN.md", href: "https://github.com/AlrikOlson/curt/blob/main/DESIGN.md" },
    },
    {
      title: "group-by + report",
      code:
        'sales = [{city:<s>"NY"</s>, amt:50}, {city:<s>"LA"</s>, amt:30}, {city:<s>"NY"</s>, amt:20}]\n' +
        '<a1>for</a1> g <a1>in</a1> sales.<a2>group</a2> .city { print <s>"{g.k} {<a3>g.v | map .amt | sum</a3>}"</s> }',
      annot: [
        { n: 1, meaning: "Grouping rows by a field is one short word (<code>group</code>) — the kind of thing that's a whole loop in most languages.", cost: "group .city · 3 tokens" },
        { n: 2, meaning: "These short built-in verbs do the work that loops spell out line-by-line elsewhere.", cost: "1 token per verb" },
        { n: 3, meaning: "The <code>|</code> pipes data left-to-right through each step — read it like a sentence.", cost: "totals per city" },
      ],
      compare: [{ lang: "curt", n: 53, win: true }, { lang: "python", n: 67 }, { lang: "go", n: 92 }, { lang: "rust", n: 102 }],
      repro: { label: "tools/tokens", href: "https://github.com/AlrikOlson/curt/tree/main/corpus" },
    },
    {
      title: "config load with rescue",
      code:
        'load p = (<a1>fs.read</a1> p).<a1>json</a1>\n' +
        'cfg = load <s>"app.cfg"</s> <a2>?</a2> {}\n' +
        'print (cfg[<s>"port"</s>] <a3>?</a3> 8080)',
      annot: [
        { n: 1, meaning: "Reading a file can fail, so curt hands back either the data or an error — and touching files needs permission, like the network.", cost: "read a config file" },
        { n: 2, meaning: "<code>?</code> means \"if that failed, use this instead\" — handling an error in a single character.", cost: "? · 1 token" },
        { n: 3, meaning: "The same <code>?</code> covers a missing setting — one simple rule for everything that can go wrong.", cost: "fall back to 8080" },
      ],
      compare: [{ lang: "curt", n: 30, win: true }, { lang: "python", n: 40 }, { lang: "go", n: 141 }, { lang: "rust", n: 100 }],
      repro: { label: "tools/tokens", href: "https://github.com/AlrikOlson/curt/tree/main/corpus" },
    },
  ];

  var title = document.getElementById("spec-title");
  var annotBox = document.getElementById("spec-annot");
  var compareBox = document.getElementById("spec-compare");
  var dots = document.getElementById("spec-dots");
  var idx = 0;

  function markup(s) {
    // <aN>…</aN> → annotation span N ; <s>…</s> → string literal
    return s
      .replace(/<a(\d)>/g, '<span class="an" data-an="$1" tabindex="0">')
      .replace(/<\/a\d>/g, "</span>")
      .replace(/<s>/g, '<span class="st">')
      .replace(/<\/s>/g, "</span>");
  }

  function render() {
    var sp = SPECIMENS[idx];
    title.textContent = sp.title;
    code.innerHTML = markup(sp.code);
    annotBox.innerHTML = sp.annot.map(function (a) {
      return '<div class="annot" data-an="' + a.n + '"><span class="marker">' + a.n +
        '</span><span class="meaning">' + a.meaning + '</span><span class="cost">' + a.cost + "</span></div>";
    }).join("");
    compareBox.innerHTML = sp.compare.map(function (c) {
      return '<div class="cell' + (c.win ? " win" : "") + '"><span class="lang">' + c.lang +
        '</span><span class="n">' + c.n + "</span></div>";
    }).join("") + '<div class="cell"><span class="lang">reproduce</span><a class="repro" href="' +
      sp.repro.href + '">' + sp.repro.label + " ↗</a></div>";
    dots.innerHTML = SPECIMENS.map(function (_, i) {
      return '<i class="' + (i === idx ? "on" : "") + '"></i>';
    }).join("");
  }

  // hover/focus links a code span to its annotation card (both directions)
  function lit(n, on) {
    document.querySelectorAll('[data-an="' + n + '"]').forEach(function (el) {
      el.classList.toggle("lit", on);
    });
  }
  function wire(box) {
    box.addEventListener("mouseover", function (e) { var t = e.target.closest("[data-an]"); if (t) lit(t.getAttribute("data-an"), true); });
    box.addEventListener("mouseout", function (e) { var t = e.target.closest("[data-an]"); if (t) lit(t.getAttribute("data-an"), false); });
    box.addEventListener("focusin", function (e) { var t = e.target.closest("[data-an]"); if (t) lit(t.getAttribute("data-an"), true); });
    box.addEventListener("focusout", function (e) { var t = e.target.closest("[data-an]"); if (t) lit(t.getAttribute("data-an"), false); });
  }
  wire(code); wire(annotBox);

  function go(d) { idx = (idx + d + SPECIMENS.length) % SPECIMENS.length; render(); }
  document.getElementById("spec-prev").addEventListener("click", function () { go(-1); });
  document.getElementById("spec-next").addEventListener("click", function () { go(1); });

  render();
})();
