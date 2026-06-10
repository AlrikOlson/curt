# cmm — a programming language measured in tokens

**Status:** design v0.2 (2026-06-10) · supersedes [v0.1](archive/DESIGN-v0.1.md) (action-DSL framing, retired by user direction) · **Working title:** `cmm` ("comm"; collides with GHC's C-- IR — naming chunk pending, though "less than C" now fits)

> `cmm` is a **general-purpose, machine-first programming language**: statically
> typed, compiled (wasm-first), capable of real software — designed for AI agents
> to write, with **output-token cost as the prime design directive** and human
> readability demoted to a derived view. The BPE tokenizer is the ISA: every
> construct is selected by measured cost, the spec ships a cost table, and CI
> fails any change that regresses it.

```cmm
handle c = for ln in c.lines { c.write ln.upper + "\n" }
for c in net.listen 8080 { go handle c }
```

A concurrent TCP uppercase-echo server. **31 tokens** (o200k_base). Python: 55.
Go: 94. Rust: 123. Same behavior, statically typed, compiles to wasm.

## 1. What the measurements actually say (read this first)

This document practices what it preaches: every number below is measured
(tiktoken, o200k_base, 2026-06-10), including the round where our own first
draft **lost**. Three real programs — word-frequency top-10, a recursive-descent
expression parser/evaluator, the concurrent echo server — written idiomatically
in cmm, Python, Go, and Rust:

| program | cmm | Python | Go | Rust |
|---|---|---|---|---|
| wordfreq | **31** | 39 (1.26×) | 155 (5.00×) | 157 (5.06×) |
| parser | **258** | 234 (**0.91×**) | 416 (1.61×) | 439 (1.70×) |
| server | **31** | 55 (1.77×) | 94 (3.03×) | 123 (3.97×) |
| **total** | **320** | 328 (**1.02×**) | 665 (**2.08×**) | 719 (**2.25×**) |

The honest reading:

1. **Versus the compiled incumbents (Go, Rust), cmm is ~2.1–2.3× cheaper to
   emit overall, and 3–5× cheaper on I/O- and structure-heavy code.** This claim
   is strong and survives scrutiny: Go/Rust ceremony (imports, signatures,
   error-handling boilerplate, types-at-every-binding) is exactly what the
   machine-first surface deletes.
2. **Versus Python, cmm reaches parity (1.02× total), not superiority.** Python
   is the empirical token floor for algorithm-shaped code: decades of
   golf-honed idioms, a hyper-dense stdlib, and no type ceremony at all. cmm
   beats it where structure dominates (1.26×–1.77×) and loses slightly where
   pure algorithm logic dominates (0.91× on the parser).
3. **The value proposition therefore is:** *Python-cost emission, plus
   everything Python doesn't give an agent* — static checking before execution,
   compilation to sandboxed wasm/native, a grammar small enough for
   constrained decoding, and deterministic single-binary deployment. Against
   Go/Rust — the languages you'd otherwise pick for those properties — the
   token savings are large and real.

### The round we lost, and what it taught

Our first draft measured **0.87× vs Python** (worse on 2 of 3 programs). The
autopsy produced two design discoveries, both now load-bearing:

- **The stdlib is the densest place to spend design effort.** Python's
  `Counter(words).most_common(10)` performs count+sort+top in ~8 tokens. No
  syntax beats a missing-by-design library call. Fix: cmm ships deliberately
  dense single-token verbs (`counts`, `top`); wordfreq went 55 → 31 tokens.
- **The ADT tax.** Type inference already deleted annotation costs, but
  *constructor ceremony* remained: `Sym "+"` cost ~4 tokens at every
  construction and pattern site. Fix: **untagged unions** (`flt | str`) with
  type-narrowing `match` — TypeScript-style discrimination, zero constructor
  tokens, still statically checked. Parser went 291 → 258.

Residual gap on the parser (258 vs 234): the type-narrowing `match` block costs
more than Python's `isinstance` trick, and irreducible arithmetic logic is the
same in every language. We report it rather than hide it; the spec chunk owns
further tournaments.

## 2. The two-sided economics

**Output side.** Output tokens are priced 3–5× input across providers in 2026
(up to 8× on reasoning models) and decode serially — emission length ≈ agent
step latency. Industry analyses call output-heavy agent patterns "the primary
cost driver in production deployments."

**Input side (the bigger lever for real software).** Agents *re-read* the
codebase: every session loads files into context; naive loops "rebill prior
context on every call." Density discounts every future read of every file.
Even on our 3-program mini-codebase, each full re-read costs **+345 tokens in
Go and +399 in Rust** versus cmm; over a 30-session engagement that's ~10–12k
input tokens on ~100 lines of code — scale to a real repository and the re-read
savings dominate the emission savings. Versus Python the input sides are equal
(328 vs 320); the win there is on the *capability* axis, not the token axis.

**KV-cache friendliness (adopted from MoonBit's LLM4Code 2024 analysis).**
Flat, linear code — toplevel definitions, minimal nesting — preserves KV-cache
prefixes across edits and lets models generate without back-navigation. cmm's
equation-based flat surface is aligned with this by construction.

## 3. Design pillars (v0.2)

**P1 — Tokenizer-as-ISA, measured forever.** Every keyword, operator, and
stdlib name verified single-token under o200k_base (Anthropic tokenizer added
in the spec chunk); a canonical corpus measured against Python/Go/Rust; CI
fails grammar changes that regress it. Character density ≠ token density
(measured: APL's `⌽⍳5` = 3 chars, 6 tokens).

**P2 — Equations + juxtaposition.** Definitions are Haskell-style equations —
`name params = body` — and calls are juxtaposition: `expr (lex s)`. No
`fn/def/func`, no parens/commas/arrows in the common path, no `return` (last
expression is the value), no `main` (toplevel statements run). The Haskell/Elm
family makes this shape abundant in model weights — density without
alienness.

**P3 — Full inference + untagged unions.** Static types everywhere, written
almost nowhere. 1-token type names (`int flt str bool`; sized `i8…u64` for
low-level work). Unions are untagged (`flt | str`) and discriminated by
type-narrowing `match` — the measured answer to the ADT tax. Optional `::`
signature lines exist only for exports/FFI boundaries.

**P4 — A deliberately dense stdlib.** Single-token, high-leverage verbs chosen
by the same measurement discipline (`counts`, `top`, `words`, `lines`,
`pairs`, `sort`, `rev`…). Admission rule: a verb earns its cheat-sheet line
only if it saves more tokens across the corpus than it costs to teach.

**P5 — Flat and linear.** Toplevel equations, one block level encouraged,
pipelines (`|`) instead of nesting, dot-chains (`x.f` = `f x`) for unary
steps. Models never balance brackets at depth, and KV-cache prefixes survive
edits (MoonBit's argument, inherited).

**P6 — Zero ceremony.** No imports (whole-program + auto-stdlib), no
visibility keywords, no semicolons (newline terminates; `;` optional inline),
**no required indentation** (measured: `"\n    "` = 2 tokens vs `"\n"` = 1;
and indentation is what models corrupt in surgical edits). Errors: postfix
`?` propagates, `expr ? fallback` rescues — both single tokens, measured 6×
cheaper than try/except blocks in v0.1.

**P7 — Identifiers: semantic, but one token.** The naming studies are
unambiguous: obfuscation measurably degrades LLM accuracy ("descriptive naming
achieving better accuracy than obfuscated"; "removing naming harms
intent-oriented tasks"), while a full word like `buf`, `idx`, `acc`, `req`
costs *exactly the same one token* as `b`. So cmm's canonical style is
meaningful single-token words; the compiler lints identifiers that tokenize
above 1. Density is spent on structure, never on gratuitous obfuscation —
that would cost accuracy and buy nothing.

**P8 — Born for constrained decoding.** The grammar is PEG/LL(1)-friendly,
ASCII-only, and ships as GBNF + llguidance/XGrammar artifacts. For a language
with zero weights presence this is the reliability floor, not a luxury: the
language-confusion literature (arXiv 2503.13620) shows models drift into
high-resource syntax (Python) when writing low-resource languages — grammar
masking makes that drift physically impossible.

**P9 — Postel parsing.** Accept the model's accent where unambiguous (`==`,
`and/or/not`, `True/None`, trailing commas, stray `return`); the canonical
formatter normalizes. Habit slips are never errors.

**P10 — Readability is a view, not a property of source.** `cmm expand`
renders dense source into an annotated projection — inferred types shown,
sugar expanded, structure prettified — for the humans who occasionally must
look. The source itself owes humans nothing; the toolchain owes them a
lens. (This is the precise sense in which cmm is "not human readable": never
*hostile*, just never *spending tokens on* readability.)

## 4. Surface tour (v0.2 draft — lang-spec-v01 finalizes)

```cmm
# equations; juxtaposition application; inference everywhere
area w h = w * h
hyp a b = (a*a + b*b).sqrt

# bindings mutate/shadow; compound ops
n = 0
n += 1

# untagged unions + narrowing match
parse w = if w[0].digit { w.flt } else { w }        # : flt | str
show v = match v { flt x -> "num {x}", str s -> "sym {s}" }

# records and 1-token types
type Pt = {x flt, y flt}
dist a b = ((a.x-b.x)**2 + (a.y-b.y)**2).sqrt

# control flow: if/else are expressions; for/while; go spawns
sign x = if x < 0 { -1 } else if x > 0 { 1 } else { 0 }

# pipelines and dot-chains; .field is a projection lambda
best = users | keep .active | top 3 .score

# errors: ? propagates, `? v` rescues
cfg = (fs.read "app.cfg").json ? {}

# destructuring; tuples
(v, rest) = expr toks

# strings interpolate; say prints
say "done: {v}"
```

No imports, no `main`, no semicolons, no required indentation, no `return`,
no visibility keywords, no lifetimes. Comments exist (`#`) and agents are
instructed never to emit them.

## 5. The worked examples (measured, both rounds)

Harness: `/tmp/cmm-tok/measure_v02*.py` this session; becomes `tools/tokens`
with committed corpus in the spec chunk. **Caveat stated plainly: no cmm
compiler exists yet — the cmm programs are validated by inspection only.**
Python/Go/Rust baselines are idiomatic-concise, not strawmen (Counter,
socketserver, gofmt-shaped Go, std-only Rust).

### wordfreq — count words, print top 10 *(31 tokens; Py 39, Go 155, Rust 157)*

```cmm
for p in (fs.read args.1).lower.words.counts | top 10 .v { say "{p.k} {p.v}" }
```

### parser — recursive-descent arithmetic evaluator *(258; Py 234, Go 416, Rust 439)*

```cmm
lex s = ((s.replace "(" " ( ").replace ")" " ) ").words | map w -> if w[0].digit { w.flt } else { w }

expr ts = {
  (v, r) = term ts
  while r.len > 0 & r[0] in "+-" {
    (v2, r2) = term r[1:]
    v = if r[0] = "+" { v + v2 } else { v - v2 }
    r = r2
  }
  (v, r)
}

term ts = {
  (v, r) = factor ts
  while r.len > 0 & r[0] in "*/" {
    (v2, r2) = factor r[1:]
    v = if r[0] = "*" { v * v2 } else { v / v2 }
    r = r2
  }
  (v, r)
}

factor ts = match ts[0] {
  flt x -> (x, ts[1:]),
  "(" -> { (v, r) = expr ts[1:]; (v, r[1:]) }
}

say (expr (lex args.1)).0
```

### server — concurrent TCP uppercase echo *(31; Py 55, Go 94, Rust 123)*

```cmm
handle c = for ln in c.lines { c.write ln.upper + "\n" }
for c in net.listen 8080 { go handle c }
```

Full per-language sources live in the measurement harness; the spec chunk
commits them as the canonical corpus.

## 6. Execution model

- **Compilation:** wasm-first (sandboxed agent execution is the native
  habitat), native via a later backend; `cmm run` JITs/interprets for
  iteration speed. Single static artifact, startup <10 ms.
- **Memory:** reference counting + cycle collector (deterministic pauses, zero
  token cost in source — no lifetimes, no `free`; NanoLang independently
  validates this choice for an LLM-targeted language). Value-type structs;
  sized integers and byte buffers for low-level work; explicit `c` FFI at
  declared boundaries (the one place `::` signatures are mandatory).
- **Concurrency:** `go expr` spawns; channels/select are spec-chunk decisions.
- **Errors:** values + `?` propagation; structured, fix-suggesting
  diagnostics designed to be fed back to the model verbatim.
- **Capability model:** no package ecosystem by design — the host grants
  capabilities (fs, net, LLM access) at startup; tool/agent affordances are a
  *library* over the host interface (`host-ffi` chunk), not grammar.

## 7. Prior art: everyone else spends tokens; cmm saves them

| project | machine-first via | token posture |
|---|---|---|
| **Vera** (Allan, 2026) | SMT-proved mandatory contracts, typed effect rows, *no names* (`@Int.0` structural refs) | **Spends heavily** — contracts + effect rows + multi-token refs at every use. "The model doesn't need to be right, it needs to be checkable." |
| **NanoLang** (Hubbard, 2026) | mandatory shadow-tests per function, Coq-proved core, unambiguous dual notation | **Spends heavily** — required test blocks ≈ double emission; explicitly keeps "humans to read" |
| **MoonBit** (2022–) | toolchain-assisted decoding, mandatory toplevel signatures, flat structure | **Spends moderately** — readable Rust-like surface; KV-cache flatness insight (we adopt it) |
| **cmm** | measured token economy + constrained decoding | **Saves** — the token is the scarce resource; verification is the host's job (tests, sandboxes), not the grammar's |

That table is the differentiation: the machine-first lane exists (Vera and
NanoLang prove it), but every occupant converts tokens *into* machine trust.
cmm bets the other way — agents already live inside verification loops
(compilers, tests, sandboxes, reviewers), so the language should make each
loop iteration as cheap as possible. Vera's bet and cmm's bet are competing
hypotheses about what actually limits agent coding (coherence vs cost); the
benchmark chunk is our falsifier. Also adjacent: CodeAct (agents should emit
code — but Python), TOON (token-efficiency for *data*, input side), Ronacher's
*A Language For Agents* (the readable-artifact lane, deliberately ceded), and
the naming/terseness studies that calibrated P7 ("Don't Force Your LLM to
Write Terse Q/Kdb Code"; variable-naming accuracy studies, 2025–26).

## 8. Rejected forms (with receipts)

- **Symbol-soup density (K/Q/APL):** character density isn't token density
  (measured), models writing terse Q/Kdb measurably degrade (practitioner
  consensus, Oct 2025), and zero-weights symbol languages amplify
  language-confusion drift.
- **Concatenative/stack (Forth):** grammar-resistant ("a static BNF grammar
  [is] inappropriate" — which kills constrained decoding), models must
  simulate stack state mentally, and control structures are non-standard.
- **Name elimination (Vera's `@Int.N`):** the naming literature shows names
  carry comprehension; structural refs also cost *more* tokens than 1-token
  names. cmm keeps names and caps their price instead.
- **Significant whitespace (Python):** +1 token per indented line and the #1
  thing models corrupt during edits.
- **Mandatory verification artifacts (contracts/tests in grammar):** valuable,
  but they belong to the host loop; cmm keeps them out of the token budget.

## 9. Risks and the kill criterion

1. **Weights gap, now with a sharper edge:** cmm has zero training presence
   AND a Python-shaped rival at token parity. Why would an agent emit cmm
   instead of Python? Answer under test: static checking catches errors
   pre-execution, wasm sandbox + capability model beats `exec(python)` for
   safety, constrained decoding guarantees parseability, and vs Go/Rust (the
   honest alternative for those properties) cmm is 2× cheaper. **Kill
   criterion (unchanged, owned by token-bench):** if model-written cmm can't
   reach success-rate within ~10pp of model-written Python after one
   language-revision cycle, we document the negative result and re-scope.
2. **Recalibrated targets** (this chunk's measurements supersede the refresh's
   provisional ≥1.3× vs Python): **parity-to-1.2× vs Python** (task-mix
   dependent, full distribution reported), **≥1.8× vs Go** (measured 2.08×
   here). The roadmap's benchmark chunk is updated accordingly.
3. **Hand-validated examples:** until interp-mvp exists, cmm programs are
   correct by inspection only. The corpus becomes executable goldens then.
4. **Comprehension risk of the dense surface:** guarded by the cheatsheet
   chunk's model-legibility QA gate (comprehension parity with Python within
   5pp or the surface gets revised).
5. **Tokenizer drift:** dual-tokenizer cost table from the spec chunk onward;
   the bet rests on the stable property (common words + ASCII ops are cheap
   in every BPE vocabulary).

## 10. Non-goals

Human writing ergonomics; package ecosystem (capabilities come from the
host); maximal theoretical density (accuracy-destroying); verification in the
grammar (the host loop owns it); replacing Python where Python is already the
token floor *and* the capability requirements are low.

## 11. Open questions for lang-spec-v01

Zero-arg call syntax under juxtaposition; mutation/value semantics at
boundaries (copy vs reference for arrays/maps); generics surface (inferred
only, or expressible?); union-narrowing exhaustiveness rules; char vs
1-string; `in` totality on unions; tuple vs record projection (`.0` vs `.k`);
`top`/`sort` argument order tournament; whether `say` or `out` survives.

---
*Sources: provider pricing analyses (Redis/Kong/digitalapplied/iternal, 2026);
Wang et al. ICML 2024 (CodeAct); toonformat.dev; Alderson token-efficiency
study (Jan 2026); Ronacher (Feb 2026); veralang.dev + aallan/vera README;
jordanhubbard/nanolang README; MoonBit LLM4Code 2024 blog; arXiv 2503.13620
(language confusion); variable-naming studies (ResearchSquare 2025, arXiv
2510.03178); "Don't Force Your LLM to Write Terse Q/Kdb Code" (HN, Oct 2025);
llguidance/XGrammar. All token counts: tiktoken o200k_base, 2026-06-10.*
