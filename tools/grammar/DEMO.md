# Constrained-decoding demo — measured results (gd-b-oss)

**Date:** 2026-06-10 · **Runtime:** llama.cpp (`llama-server`, Metal, brew) ·
**Model:** cognitivecomputations Dolphin-class 8B (local GGUF, 3.8 GB quant) —
an OSS model with **zero cmm in its weights**: cmm is a zero-weights language
and grammar masking is its reliability floor.

## The numbers

| arm | n | parse-valid | **mid-stream violations** |
|---|---|---|---|
| **8B + cmm.gbnf mask** | 200 | 169 (84.5%) | **0** |
| 8B unconstrained (same prompts) | 100 | 44 (44.0%) | 55 |
| Claude Haiku, cmm-naive (no tools) | 10 | 7 | 3 |
| Claude Haiku, self-taught from repo artifacts | 10 | **10** | 0 |

Every sample validated by `cmm parse` — the reference parser is the oracle.

**The soundness claim, mechanically classified:** all 31
constrained failures are **valid program prefixes** (the parse error is
exactly at Eof) — the mask never permitted a single mid-stream syntax
violation in 200 generations. Failures are pure *non-termination*:
the model wanders legally inside unbounded header sub-languages
(`while 1 1 1 …`) until the token cap (38 budget-doubling
retries fired), plus one measured **llama.cpp EOS-escape** (the engine
permits EOS mid-grammar: `record 2-point == {` stopped "complete").
Unconstrained failures are the opposite: 55/56 are real mid-stream Python
drift (`while i > 0: …`, comprehensions, `match…case`).

## What this run found and fixed (chronological)

1. **Keyword widening leaked 30%** (run 1: 140/200). GBNF's NAME originally
   included keywords; the model's Python prior drove `[x for x in …]` /
   `while…do` straight through. → `lark2gbnf.py` now GENERATES exact keyword
   exclusion as a prefix-trie complement (no lookahead needed). Permanent
   artifact win.
2. **Engine state explosion**: bounding starred groups `(X)* → (X)? (X)?`
   hung llama.cpp's mask evaluation (>300 s/request). Reverted; termination
   pressure is a +4.0 logit bias on the newline token — sound, because bias
   only reorders within the mask's legal set.
3. **The termination problem**: masks guarantee validity of *completed*
   outputs, not completion. A weak model can wander legally until the cap.
   Production guidance: engines must expose grammar-final-state
   (llguidance-class) or deployments must verify completion; llama.cpp's
   EOS-escape (finding above) makes `stop_type` insufficient.
4. **A real three-way grammar divergence, found by generation**: PEG, Lark,
   and GBNF all allowed `if`/`match` expressions as application *arguments*
   (`print if c {1} else {2}`); the Rust parser allows them only at
   expression head. Never exercised by the corpus — only masked GENERATION
   surfaced it. All three grammars tightened (arg0/arg split); negative
   agreement suite extended to 12/12.

## The Haiku rows

Claude Haiku cannot be grammar-masked (Anthropic exposes JSON-schema
structured outputs only — no arbitrary CFG as of mid-2026), so it ran
unconstrained with the same one-line system framing the 8B got. Naive: 7/10,
failing in the same Python-drift shapes as the 8B. Given repo access, Haiku
**self-taught cmm from SPEC.md and the corpus in ~12 tool calls and scored
10/10 idiomatic lines** — strong early evidence for the cheat-sheet thesis
(the next chunk measures exactly that, properly).

## Reproduce

```sh
cargo build --release
tools/grammar/constrained_demo.py --model <gguf>   # writes results.json
tools/grammar/mkdemo.py                            # renders this file
```

Demo-grammar deltas (applied at runtime, committed artifact untouched):
`root ::= stmt "\n"` (single statements), whitespace pruned to single
spaces, +4.0 newline logit bias, one budget-doubling retry on truncation.

## Capability matrix (as of 2026-06-10)

| surface | arbitrary CFG? | status for cmm |
|---|---|---|
| llama.cpp (GBNF) | yes (EOS-escape caveat, measured) | **0 mid-stream violations / 200 gens; 84% parse-valid incl. non-termination** |
| llguidance / vLLM (Lark) | yes | artifact ready (`cmm.lark`); demo pending a vLLM host |
| OpenAI custom tools (Lark/regex CFG) | yes; conformance not guaranteed (community, Aug 2025) | **blocked: no `OPENAI_API_KEY`** (chunk gd-b-openai) |
| Anthropic Structured Outputs | JSON-schema only (GA) | whole-program cmm masking not possible as of mid-2026 — Haiku measured unconstrained above |
