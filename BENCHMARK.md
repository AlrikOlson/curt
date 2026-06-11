# curt token-bench — the moment of truth

**Date:** 2026-06-11 · **Design:** 15 held-out tasks (disjoint from corpus/
and the cheatsheet experiment) × 4 languages × 2 models (claude haiku,
claude sonnet subagents) × 3 samples, single-shot generation, then ONE
diagnostics-fed repair cycle for the kill-criterion pair (curt, Python).
Grading is mechanical (`tools/bench/grade_bench.py`): execute, compare
stdout to frozen `.expected` (numeric tokens compared numerically so
`86` ≡ `86.0` across languages). All 360 generated + 24 repaired programs
are committed under `tools/bench/answers/` — every cell regradeable.

Context protocol (measured in the cheatsheet experiment): the curt lane
gets CHEATSHEET.md (~1.6k o200k tokens); Python/Go/Rust get name-only —
their "sheet" is their training data. The asymmetry is the point being
tested and is stated, not hidden.

## Success rate (solved / 45 cells per model·language)

| language | haiku 1-shot | sonnet 1-shot | haiku +repair | sonnet +repair |
|---|---|---|---|---|
| Python | **45/45** | **45/45** | 45/45 | 45/45 |
| Go | 45/45 | 45/45 | — | — |
| Rust | 44/45 | 45/45 | — | — |
| **curt** | **29/45 (64%)** | **37/45 (82%)** | **41/45 (91.1%)** | **44/45 (97.8%)** |

Go/Rust got no repair round (kill criterion compares curt to Python; both
lanes were ~ceiling anyway).

### Kill criterion — curt within ~10pp of Python after one revision cycle

| model | gap after repair | verdict |
|---|---|---|
| haiku | 8.9pp | **PASS** (borderline) |
| sonnet | 2.2pp | **PASS** |

Honest caveats, in decreasing order of weight:

1. **Single-shot, curt FAILS the 10pp bar on both models** (−35.6pp
   haiku, −17.8pp sonnet). The pass is bought by the revision cycle.
2. **The repair prompt was diagnostic + corrected docs**, not diagnostic
   alone: it included a reminders addendum of verified v0.1 facts the
   frozen sheet lacks (no list `+`/`+=` — append is `[xs,[x]].flat`; no
   `.contains` — use `in`; `sort`/`max` take no key function; `.pairs`
   gives `.k`/`.v`). Those facts are sheet-v2 content; with them in the
   sheet, some single-shot failures likely move. Unmeasured — flagged.
3. **One unrepaired cell is an interpreter bug, not a model error** (see
   below). Sonnet's repair was documentation-correct; the toolchain is
   the oracle, so the cell stays failed.

## Output tokens (o200k_base, solved single-shot cells)

| | curt | Python | Go | Rust |
|---|---|---|---|---|
| median tokens | **58** | 70 | 102.5 | 105 |
| per-task median ratio vs curt | — | **1.00×** (0.76–2.05, n=14) | **1.50×** (1.17–3.29) | **1.52×** (0.88–3.08) |

**Targets NOT met — honest negative.** The chunk targeted ≥1.3× vs
Python and ≥1.8× vs Go. Measured: **parity with Python (1.00×)** and
1.5× vs Go. The corpus cost table (reference code: 1.19× / 2.34×) does
not transfer to model-generated code: models write compact idiomatic
Python but verbose, loop-heavy curt — they don't yet reach for the
verbs that make curt cheap (`counts`, `top`, projections), or they
reach for them and miss (see failures). Token efficiency of a language
is a property of *the code models actually emit in it*, not of its
reference corpus. n=14 (05_dedup excluded: never solved in curt
single-shot).

`14_rect_lib` re-read cost (median tokens of solved solutions — the
price of holding the module in context): **curt 95** · Python 108.5 ·
Go 125 · Rust 157. curt cheapest, 1.14× vs Python — direction right,
magnitude small at this module size.

## What broke (curt failure anatomy, 24/90 single-shot failures)

| cause | cells | class |
|---|---|---|
| no list append (`+`/`+=` on lists) | 6 (05_dedup, all lanes) + 2 elsewhere | missing v0.1 surface |
| invented verbs/forms (`contains`, `sort`-with-key, `max`-with-key, `'a'` char literals, `++`, `continue`, `split ""`) | 8 | sheet/lang gap |
| pipe-captures-last-argument | 3 | the known footgun, 3rd experiment in a row |
| multiline list literal (newline after `[`) | 2 | grammar ergonomics |
| `.pairs` field names guessed `.0/.1` | 2 | sheet gap |
| `fs` variable name collides with capability namespace | 1 | naming hazard |

**Interpreter bug found by the benchmark:** at binding position, pipe
capture reaches *inside* a parenthesized first stage —
`total = (s.split ",") | map f | sum` fails with `, is not a list` even
though the cheat sheet prescribes exactly this wrap. The wrap only works
when the pipeline is itself a call argument (`print (s.split ",") | ...`).
Repro: `tools/bench/answers/curt_sonnet_r1/s3/10_parse_pairs.curt`.
Filed to the spec-truth chunk.

## Positioning vs published 2026 measurements

Martin Alderson's token-efficiency ranking (martinalderson.com, Jan
2026) measures 19 *existing* languages on *static human-written*
RosettaCode corpora with no execution, finding a 2.6× spread. This
benchmark asks the adjacent question that ranking can't: what do models
*generate* in a *designed* language, does it run, and what does it cost?
Same instrument (tokenizer), different object (generated + executed
code, success-rate-weighted). Result: the generated-code lens is harsher
— curt's reference-corpus advantage over Python (1.19×) evaporates to
parity on model-written code. SWE-AGI (arXiv 2602.09447) independently
validates the contamination logic of benchmarking on a nascent language.

## Verdict

The bet ("output-token cost is the ISA") is **not yet paying off against
Python** on model-generated code: token parity at a success-rate
deficit. It IS paying off against Go/Rust (1.5×, at much higher curt
repair-adjusted success than the targets assumed possible). The kill
criterion formally PASSES (both models within 10pp after the allowed
revision cycle), so the project proceeds — but the honest reading is
that v0.2 must close the failure anatomy above (list append, the
pipe-capture semantics, the interpreter bug) and put the missing facts
into the sheet, or the Python gap will not close. The revision content
is now fully measured, not speculative.

## Post-revision re-run (spec-truth, 2026-06-11)

The one revision cycle was implemented (`spec-truth`: list `+`, paren
capture barrier, two-arg `range`, newline-in-brackets, mixed-list unions,
Postel `++`/`'x'`, capability shadowing, CHEATSHEET v2) and the curt lanes
re-ran on the SAME frozen tasks with fresh single-shot generations
(`answers/curt_*_v2/`):

| | v0.1 + sheet v1 (1-shot) | v0.1-revised + sheet v2 (1-shot) | Python (1-shot) |
|---|---|---|---|
| haiku | 29/45 (64%) | **37/45 (82%)** | 45/45 |
| sonnet | 37/45 (82%) | **45/45 (100%)** | 45/45 |

**Sonnet reaches Python parity on success with NO repair round.** Haiku
gains +17.8pp single-shot; its residual failures are now idiom-invention
(sort-with-key variants on 09_topwords ×3, `else` placement) rather than
missing language surface. Caveat: the delta measures language fixes +
sheet v2 jointly (the revision changed both, deliberately — that was the
chunk). Token side: unchanged at parity (python/curt per-task median
0.95×, n=15, vs 0.96× before) — the success gap closed without
spending tokens, but the targeted 1.3× token advantage over Python
remains future work (idiom density: models still write loop-heavy curt).
Reference-corpus median improved 1.19× → 1.12× (n=21) with the new
append-exercising corpus program.

## The idiom-density experiment (2026-06-11): hypothesis tested, half refuted

Pre-registered hypothesis (roadmap `idiom-density`, from full-text reads of
CangjieBench 2603.14501, ShortCoder 2601.09703, CL4SE 2602.23047): idiom
density is a style that transfers via within-language cost-contrast pairs;
where it doesn't, a verifier-backed densifier (`curt dense`) banks the
token win mechanically. Four arms on the frozen tasks, 2 models × 3
samples each:

| arm | context/processing | solved/90 | py/arm per-task ratio |
|---|---|---|---|
| A | sheet v2 (prose rules) | 82 | 0.95× |
| B | sheet v3 (rules + 3 verified cost-contrast pairs, −10 tok budget) | 81 | 0.94× |
| C | A + post-hoc `curt dense` | 82 | **1.01×** |
| D | B + post-hoc `curt dense` | 81 | 0.99× |

**Leg 1 REFUTED:** in-context cost-contrast pairs did not move idiom
density (0.94× vs 0.95×, noise) — and made sonnet's pipe usage MORE
eager, reintroducing the pipe-capture slip on 10_parse_pairs in all
three samples (the footgun's fourth consecutive experiment). This
refines CL4SE: their style-transfer-via-exemplars finding (NL summaries)
does not extend to token-level code idioms in a pretraining-absent
language at a 3-pair budget.

**Leg 2 CONFIRMED but narrow:** `curt dense` preserved success exactly
(the differential gate held: 0 divergences across 225 committed programs
+ all arms) and recovered +6pp of token ratio — but rewrite library
R1–R4 only touches single-statement loop bodies (13/225 programs).
Magnitude is bounded by the library, not the mechanism.

**The reframe the data forces:** even the maximally idiomatic reference
solutions sit near Python parity on these tasks. The reference-corpus
advantage (1.12× median) is concentrated in CEREMONY domains — errors
(1.38×), concurrency (1.75×), serving (1.72×), records (1.27×) — which
this benchmark's algorithm/text task mix undersamples. **The token bet
is domain-shaped, not style-shaped**: where Python is terse, curt ties;
where Python (and especially Go/Rust) pay ceremony, curt wins. A
domain-weighted benchmark is the honest next measurement.

Also measured during pair selection: pipeline forms with lambdas cost
the same as the loops they replace (sumsq pair delta = 0); the real
token wins are lambda-free verb forms (`.max`, `join`, `top n .f`).

## Reproduce

```
cargo build --release
.ci-venv/bin/python tools/bench/grade_bench.py --all   # every cell
.ci-venv/bin/python tools/bench/measure.py             # token table
```

Generation/repair are LLM calls (2026-06-11, haiku + sonnet subagents);
the committed answers/ are the frozen record — grading is deterministic.
