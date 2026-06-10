# tools/grammar — constrained-decoding artifacts (gd-a)

Grammar-masked generation is cmm's reliability floor: a zero-weights language
gets zero parse errors by construction (models drift toward Python syntax;
the mask makes that impossible). Refreshed landscape (2026-06-10, think:15):
OpenAI's custom tools accept arbitrary CFGs in **Lark** syntax, and
**llguidance** (the engine behind several vendors' structured outputs) also
consumes Lark — so one artifact feeds both. llama.cpp uses **GBNF**.

## Artifacts

- **`cmm.lark`** — the PRIMARY artifact: a CFG twin of
  `tools/tokens/grammar.peg` with **explicit whitespace** (no `%ignore`).
  Explicit ws is load-bearing: SPEC §1 adjacency is semantic (`x.f` vs
  `x .f`, `x?` vs `x ? y`, glued `Pt{`/`f(`/`xs[`), and only an explicit-ws
  grammar preserves it. Earley-parsed for validation; membership is what
  constrained decoding needs.
- **`cmm.gbnf`** — GENERATED from `cmm.lark` by `lark2gbnf.py`. Never
  hand-edit. Known, documented widening: GBNF has no lookahead, so `NAME`
  cannot exclude keywords there (over-acceptance only — a mask that
  over-accepts never blocks a valid program; the Rust parser stays the
  oracle).
- **`lark2gbnf.py`** — mechanical, deterministic converter. Terminal regexes
  are pinned in `TERMINAL_MAP`; if `cmm.lark` changes a terminal, conversion
  fails loudly instead of drifting.
- **`validate.py`** — the divergence gate (CI): `cmm.lark` must parse the
  golden corpus **20/20** AND agree with the Rust parser (`cmm parse`) on a
  negative sample set (**10/10** invalid snippets rejected by both). Exit 0
  only when both hold.

## Honest derivation note

PEG ordered choice is not mechanically transformable to a CFG. The Lark twin
is kept honest the same way `grammar.peg` itself is — by the machine gate,
not by generation. The chain of trust: Rust parser (oracle) ⇄ grammar.peg
(PEG gate) ⇄ cmm.lark (this gate) → cmm.gbnf (generated, pinned terminals).

## Running

```sh
cargo build --release            # the oracle
python3 tools/grammar/validate.py    # needs `lark` (pip install lark)
python3 tools/grammar/lark2gbnf.py   # regenerate cmm.gbnf
```

## Next (gd-b)

OSS-runtime zero-error demo (llama.cpp + `cmm.gbnf`, ≥200 generations),
OpenAI custom-tools conformance run (≥100 generations, measured — community
reports conformance is not guaranteed), capability-matrix re-check, and the
constrained-vs-unconstrained quality comparison. Needs a local model and
`OPENAI_API_KEY`.
