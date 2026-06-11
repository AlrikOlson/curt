# domain-bench — the token bet measured where it was supposed to live

**Date:** 2026-06-11 · **Question:** idiom-density's reframe (BENCHMARK.md)
claimed the token bet is domain-shaped — curt's reference-corpus advantage
concentrates in CEREMONY domains (errors 1.38×, records 1.27× vs Python).
Does that survive model-GENERATED code? **Pre-registered bars: ≥1.3× vs
Python and ≥2× vs Go/Rust per ceremony domain, at no success regression.**

**Design:** 10 held-out tasks in 4 deterministic ceremony domains — errors
(e1–e3), records/JSON (r1–r3), fs (f1–f2), multi-step validation (m1–m2) —
× 4 languages × 2 models (haiku, sonnet) × 3 samples, single-shot.
curt lane: CHEATSHEET v3 in context, `--fs` granted. Twins: name-only.
Rust: std-only (no serde) — flagged below. Fixtures + frozen `.expected`
(run-verified Python oracles) in this directory; grading mechanical
(`grade_dbench.py`, cwd=fixtures, numeric-normalized). Spawn and serving
EXCLUDED (nondeterministic under byte-equality grading) — their corpus
ratios (1.75×, 1.72×) remain unverified on generated code.

## Success (solved / 60 per language)

| language | total | haiku | sonnet |
|---|---|---|---|
| Python | 60/60 | 30/30 | 30/30 |
| Go | 60/60 | 30/30 | 30/30 |
| Rust | 56/60 | 26/30 | 30/30 |
| **curt** | **36/60 (60%)** | **10/30 (33%)** | **26/30 (87%)** |

**curt's success COLLAPSES in its supposedly winning domains.** The same
haiku that scored 82% on the algorithmic bench scores 33% here; sonnet
drops from 100% to 87%. The io/error surface gets one paragraph in the
sheet while the bench tasks lean on it entirely.

## Tokens (o200k, per-task median over solved cells, ratio = lang/curt)

| domain | py/curt | go/curt | rust/curt |
|---|---|---|---|
| errors | 1.03× | 1.66× | 1.73× |
| records | 1.00× | **2.79×** | **6.72×** |
| fs | 0.86× | 2.00× | 1.64× |
| multi | 0.96× | 2.33× | 3.61× |
| **overall** | **0.98×** | **2.46×** | **3.83×** |

Best single task: `e2_missing_file` (read-missing-config-with-fallback) —
1.46× vs Python, 3.85× vs Go, 7.20× vs Rust. Worst: `f2_adults` 0.81× vs
Python.

## Verdict per pre-registered bar

| bar | result |
|---|---|
| ≥1.3× vs Python in ceremony domains | **FAIL — every domain.** 0.86–1.03×. Only one task (e2) reaches 1.46×. |
| ≥2× vs Go in ceremony domains | **PASS in records (2.79×), fs (2.00×), multi (2.33×); errors 1.66× falls short.** |
| ≥2× vs Rust | PASS in records/multi (6.72×/3.61×); errors/fs short. Caveat: std-only Rust hand-parses JSON — serde would shrink records/r-tasks substantially. Treat Rust ratios as upper bounds. |
| no success regression | **FAIL — the opposite: −40pp vs the algorithmic bench.** |

**The domain-shaped hypothesis is REFUTED against Python on generated
code.** Python's try/except + dict.get + f-strings are as terse as curt's
rescue/`?` at these program sizes; the corpus's 1.38× errors advantage
came from reference style, not from anything models reproduce. The bet
survives — strongly — only against Go/Rust ceremony.

## Failure anatomy (24 curt failures)

| cause | cells | note |
|---|---|---|
| unparenthesized lambda pipe stage (`<fn> is not a list/has no len`) | 7 | lambda swallows `\|` — 5th consecutive experiment for the pipe family |
| `err` used as a value/pattern (`err is not defined`) | 5 | error-surface gap: T \| err has no expressible `err` literal |
| int/float comparison (`o["amt"] > 25` silently empty) | 3 | strict no-mixing turns a type error into wrong OUTPUT — worst failure mode in the language |
| interpolation misuse (`{}`, quotes in holes) | 4 | |
| other (scope/parse slips) | 5 | |

## What this means

1. **The launch claim must be**: parity with Python on tokens AND success
   (algorithmic bench), 1.5–2.5× cheaper than Go and 1.5–4× cheaper than
   Rust on model-written code — NOT "beats Python". Three experiments
   (wording, contrast pairs, domain-weighting) failed to surface a
   Python token win; that hypothesis line is exhausted at v0.1.
2. **Success in io/error domains is the live problem**: the sheet's io
   coverage is one paragraph; and two language-level footguns (lambda-
   swallow in pipes, silent int/float comparison) plus one surface gap
   (`err` not expressible) account for 15/24 failures and are fixable.
3. Rust ratios need a serde-allowed re-run before being quoted.

## Reproduce

```
cargo build --release
.ci-venv/bin/python tools/dbench/grade_dbench.py --all
```

Generations are LLM calls (2026-06-11, haiku/sonnet subagents); the
committed answers/ are the frozen record — grading is deterministic.

## Post-fix re-run (v02-footguns, 2026-06-11)

The four language flaws were fixed (v0.2: pipe/rescue take the whole left
expression — capture DELETED; lambda bodies stop at `|`; `err` is both a
match pattern and a constructor (`err "msg"`), equality compares err
values; maps answer field syntax with key lookup; block lambdas keep
newlines inside call parens; `'...'` raw strings; `"{}"` literal) and the
curt lanes re-ran fresh on the SAME frozen tasks with sheet v4
(`answers/curt_*_v2/`):

| | v0.1 lanes | v0.1 lanes regraded under v0.2 | fresh v0.2 lanes |
|---|---|---|---|
| haiku | 10/30 | 18/30 | **21/30** |
| sonnet | 26/30 | 29/30 | **30/30** |
| total | 36/60 (60%) | 47/60 | **51/60 (85%)** |

**The frozen-code column is the purest measurement**: the same committed
model programs gain +11 cells with zero regeneration — the failures were
the language's, not the models'. One frozen cell regressed (a
`print x ? y` statement-level rescue that old capture made work; the
checker now rejects that form loudly). Fresh sonnet reaches **100%**;
fresh haiku's 9 residual failures are idiom inventions (list patterns in
match, logic slips) — model skill, not language surface.

**The three target failure classes are at ZERO** in the fresh lanes:
no lambda-swallow, no err-inexpressibility, no silent comparison
wrongness. Tokens: fresh lanes at 1.01× vs Python (parity maintained;
no token cost paid for the fixes). The 45-task algorithmic bench shows
NO regression (sonnet v2 45/45 unchanged; the frozen sonnet v3 lane
IMPROVES 42→45 — its three pipe-capture failures were correct code the
old language broke).
