# Cheat-sheet marginal-value experiment — results

**Date:** 2026-06-11 · **Question:** what is CHEATSHEET.md worth per context
token, against (A) nothing and (B) the raw artifacts (SPEC.md + corpus)?
All grading mechanical (`grade.py`): `curt parse` / `curt check` /
`curt run` vs frozen `.expected`; the three gates are independent.
Generation was single-shot (no tool use, no self-repair) by fresh
subagents; arm B agents were allowed exactly one command
(`cat SPEC.md corpus/*.curt`) to load the artifacts into context.

## Context costs (o200k_base, measured)

| arm | context | tokens |
|---|---|---|
| A | name + one-liner | ~25 |
| B | SPEC.md (4,635) + 20 corpus .curt (944) | **5,579** |
| C r1 | CHEATSHEET.md as of round 1 | **1,602** |
| C r2 | CHEATSHEET.md after wording iteration | **1,634** |

Anthropic-tokenizer column: **absent** — no `ANTHROPIC_API_KEY` in this
environment; o200k_base is the primary scale as throughout the project.

## Generation matrix (10 held-out tasks; parse / check / run out of 10)

| arm | model | parse | check | run |
|---|---|---|---|---|
| A (~25 tok) | haiku | 0 | 0 | 0 |
| A (~25 tok) | sonnet | 1 | 1 | 1 |
| B (5,579 tok) | haiku | 9 | 6 | 5 |
| B (5,579 tok) | sonnet | 9 | 6 | 5 |
| **C r1 (1,602 tok)** | haiku | 10 | 8 | **8** |
| **C r1 (1,602 tok)** | sonnet | 10 | 9 | **9** |
| C r2 (1,634 tok) | haiku | 9 | 7 | 7 |
| C r2 (1,634 tok) | sonnet | 10 | 9 | 8 |

Raw model outputs are committed under `answers/`; regrade any cell with
`grade.py answers/<arm>_<model>`.

## Marginal-value verdict

**The sheet beats the raw artifacts at ~3.5× fewer context tokens**
(headline r1 sheet: 5579/1602 = 3.5×; current r2 sheet: 3.4× per
`measure_arms.py`).
Run success, pooled across both models: A 1/20 · B 10/20 · C r1 17/20.
Per context token, the sheet delivers ~4.7× arm B's success
(17/1602 vs 10/5579, taking arm A's ~1/20 as the zero-context floor).
The hypothesis was C ≈ B at fewer tokens; the measured result is
**C > B** — the sheet states the traps (lambda-swallows-pipe, `if` can't
sit bare as an argument, one-arg `range`) that SPEC + corpus never make
explicit. Both B failures on 07 were the unparenthesized-lambda pipe
stage; both B failures on 06 were `print if ...`; the sheet's C arms
avoided both.

**Wording iteration was a wash (honest negative).** Round 2 strengthened
the pipe-capture warning ("WRONG", an explicit rule line) after both C r1
models failed 08_csv_sum on exactly that trap. Result: 08 still failed in
both r2 arms, and unrelated single-sample variance (haiku misread 01;
sonnet invented ASCII arithmetic on 05) moved totals **down** 1 point
each. Two conclusions: (1) at n=10 single-shot, ±1–2 points is noise;
(2) the pipe-captures-last-argument footgun **resists prose warnings** —
models pattern-match `x.split "," | ...` from other languages' pipe
semantics regardless of instruction. This is design feedback for v0.2,
not a documentation bug. Iteration stopped at 2 of the allowed 3 rounds;
a third would be noise-chasing.

## Legibility QA (predict exact stdout; 10 curt/Python twin pairs)

| model | curt | Python | delta |
|---|---|---|---|
| haiku | 10/10 | 9/10 | +10pp |
| sonnet | 8/10 | 9/10 | −10pp |
| **pooled** | **18/20** | **18/20** | **0pp** |

Within the 5pp acceptance band pooled. The only failure both languages
shared (Q08, `half(half(10))`) was an arithmetic slip — three of four
arms made the identical mistake in both languages, so it is not a curt
legibility cost. Sonnet's curt-only miss (Q01: 100 vs 200) is the single
genuinely curt-attributable comprehension error in the set.

## Toolchain divergences surfaced by the experiment (design feedback)

1. **Map literals don't exist.** SPEC §3 calls `{K: V}` string-keyed
   literals the default; grammar.peg and the interpreter reject them;
   the corpus never exercises one. Sheet teaches implemented truth.
2. **`range a b` is broken.** Documented in SPEC §5 and the §12
   tournament record, but mis-elaborates everywhere (`range` resolves at
   arity 1, surplus re-nests: `1 is not callable`). Corpus only uses
   `range n`. Arm B models — taught by SPEC — wrote `range 1 16` and
   parsed but failed; the sheet teaches one-arg range.
3. **check/run disagree on mixed-list unions.** `[7, "ok", 12]` fails
   `curt check` (no union inference for list literals) but runs fine.
   `grade.py` grades the gates independently so this shows up instead of
   being masked.
4. **Pipe-captures-last-argument** is the dominant model footgun (every
   arm, both models, both rounds hit it at least once).

## Reproduction

```
cargo build --release
.ci-venv/bin/python tools/cheatsheet/experiment/grade.py \
    tools/cheatsheet/experiment/answers/C_sonnet        # any arm
.ci-venv/bin/python tools/cheatsheet/experiment/measure_arms.py
```

Generation is LLM-dependent (subagent calls, models `haiku`/`sonnet`,
2026-06-11); the committed `answers/` are the frozen record of those
generations — grading them is fully deterministic.
