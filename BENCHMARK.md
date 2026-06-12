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

## Tokenizer robustness (tokenizer-truth, 2026-06-11)

Every ratio above was re-measured across four tokenizers
(`tools/tokens/sensitivity.py` — one command reproduces this table):

| tokenizer | corpus py/curt | bench py/curt | dbench py/curt | fragmenting verbs |
|---|---|---|---|---|
| o200k_base | 1.12× (n=21) | 0.94× (n=15) | 1.01× (n=10) | none |
| cl100k_base | 1.10× (n=21) | 0.94× (n=15) | 1.01× (n=10) | none |
| qwen2.5-coder | 1.09× (n=21) | 0.94× (n=15) | 1.01× (n=10) | none |
| deepseek-coder | 1.19× (n=21) | 1.01× (n=15) | 1.09× (n=10) | counts(2), pairs(2), chars(2) |

**The claims replicate**: per-task ratios agree within ~0.03× across
o200k/cl100k/Qwen; DeepSeek's vocabulary is slightly MORE favorable to
curt despite fragmenting three verbs (`counts`/`pairs`/`chars` are 2
tokens there — listed, not renamed; the corpus-level effect is positive).
Every stdlib verb and operator is a single token in bare, dotted, and
piped positions on o200k, cl100k, and Qwen2.5-Coder. The Anthropic
count-tokens lane is implemented and key-gated (`ANTHROPIC_API_KEY`);
run it before quoting Anthropic-specific numbers.

## Long-form exhibit (flagship-logmill, 2026-06-11)

Every program above is short (the corpus maximum was 28 lines). The
flagship, [corpus/22_logmill.curt](corpus/22_logmill.curt), is the
first long-form datapoint: 126 lines, a JSON-driven log-analytics
engine exercising 18 constructs in one coherent program, with four
golden invocations pinning every spec-resolution path.

| program | curt (o200k) | python twin | py/curt |
|---|---|---|---|
| 22_logmill (126 lines) | 982 | 1033 | **1.05×** |

The corpus median moves 1.12× → **1.10× vs Python (n=22)**. The
flagship's own 1.05× sits *below* that median — consistent with the
idiom-density finding above: curt's density advantage is concentrated
in ceremony domains (errors, I/O, records), and a long program dilutes
those with identifier-heavy report lines that cost the same in any
language. Published as measured.

Writing the flagship also surfaced two checker/runtime coherence gaps
(record-type match arms never match at runtime; sig-declared `T | err`
defeats match exhaustiveness), filed as roadmap chunks
`match-recordarm` and `sig-err-any` — finding these was a goal of the
exercise, not a footnote: nothing this large, and no program combining
more than 3 curt-native features, had ever been written.

## Loop dollars (agent-loop-bench, 2026-06-12)

Everything above prices the *program text*. Agents do not pay for
program text; they pay for **task loops** — generate, run, read the
failure, repair — and most of a loop's tokens are input (context
re-reads), not output. The pre-registered, falsifiable hypothesis:
curt's loop cost beats Python's at generation parity, because its
single-line diagnostics and compact source make every turn after the
first cheaper. Published whichever way it falls.

**Protocol.** 25 frozen tasks (15 compute, 10 file-driven), two models
(claude-haiku-4-5 at $1/$5 per MTok in/out, claude-sonnet-4-6 at
$3/$15), curt vs Python: one generation plus up to two repair turns;
native failure surfaces fed back verbatim (curt: the single-line JSON
diagnostic; Python: the traceback tail); wrong-output feedback shows
the program's *own* stdout only — the expected output is never
revealed. Temperature 1.0, one sample per cell, n=100 cells. System
prompts: curt carries its agent cheat sheet (2,034 o200k tokens);
Python carries an 18-token one-liner (it needs no teaching — that
asymmetry is the honest cost of being a new language, so it is priced
in, not excused away). Prompt caching requested identically on both
system blocks; cache writes priced at 1.25×, reads at 0.1×. All 100
transcripts frozen verbatim, failures included; the matrix below
re-derives deterministically from the frozen files. Total API spend:
$0.3282.

| model | lang | solved | $ total | $/solved | turns (solved) | input tok | output tok | diag tok |
|---|---|---|---|---|---|---|---|---|
| haiku | curt | **22/25** | $0.1059 | $0.0048 | 1.23 | 87,501 | 3,686 | 398 |
| haiku | py | 19/25 | $0.0298 | **$0.0016** | 1.05 | 6,366 | 4,689 | 665 |
| sonnet | curt | **22/25** | $0.1012 | **$0.0046** | 1.05 | 76,913 | 2,920 | 238 |
| sonnet | py | 19/25 | $0.0913 | $0.0048 | 1.05 | 6,963 | 4,691 | 766 |

**What curt won, measured:**

- **Success: 22/25 vs 19/25 on both models.** On haiku the mechanism
  is isolable: first-shot success was *identical* (18/25 each); curt's
  entire +3 came from the repair loop, which converged in 4 of 7
  repair-entered cells vs Python's 1 of 7. The verbatim-`fix`
  diagnostic is doing exactly the work it was designed to do.
- **Output tokens: 21–38% fewer** (3,686/2,920 vs 4,689/4,691) — the
  static corpus advantage survives live generation.
- **Feedback compactness: 57.8 vs 102.2 tokens per repair cell**
  (curt 636 over 11 cells; Python 1,431 over 14) — the single-line
  diagnostic is ~44% smaller than a traceback tail in practice.

**What curt lost, measured:** on haiku, $/solved is **3× worse**
($0.0048 vs $0.0016). The cache fields in the frozen transcripts give
the mechanism exactly:

| lane | uncached in | cache writes | cache reads |
|---|---|---|---|
| haiku curt | 87,501 | **0** | **0** |
| sonnet curt | 2,345 | 8,100 | 66,468 |

The cheat sheet is ~2.3k tokens on the wire — *under* haiku's 4,096-
token minimum cacheable prefix (so it is re-billed at full price every
single call: 87.5k input tokens, $0.0875 of the lane's $0.1059) and
*over* sonnet's 2,048 floor (so 86% of the same lane's input arrives
as 0.1× cache reads, and curt narrowly wins $/solved). Python's loop
cost is output-dominated; curt's is input-dominated — and the input is
not the program or the diagnostics, it is the documentation.

**Verdict on the hypothesis: split, and the split is the finding.**
Refuted on haiku — the documentation tax, paid uncached, dwarfs every
per-turn saving. Narrowly confirmed on sonnet — with the sheet cached,
curt delivers more solved tasks at a lower price per solve. The
decisive variable for a new language's loop economics is not
diagnostic size (a second-order lever measured in hundreds of tokens)
but **teaching-context cache economics** (tens of thousands): whether
the model's cache floor swallows your cheat sheet decides the sign of
the result. Caveats, stated plainly: one sample per cell at
temperature 1.0, so per-task results carry sampling noise (the +3
success margin repeating across both models is the strongest signal
here); two file-driven tasks (`f2_adults`, `m1_oldest`) failed in all
four lanes — task difficulty, not language.

Reproduce: `.ci-venv/bin/python tools/bench/loop.py report`
(deterministic over `tools/bench/loops/*.jsonl`; re-running the live
matrix requires an API key and ~$0.33).
