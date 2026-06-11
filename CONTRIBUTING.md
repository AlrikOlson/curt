# Contributing to curt

curt is a machine-first language built on one discipline: **every claim is
measured, never estimated**. Contributions live or die by the same rule.

## The doctrine

1. **Measured, never estimated.** Any token-cost claim comes from a real
   tokenizer run (`tools/tokens/count.py`, o200k_base). If you state a
   number, a script in this repo must reproduce it exactly.
2. **Tournament before adoption.** New syntax or stdlib verbs are measured
   against alternatives on the corpus (`tools/tokens/tournaments.py`). The
   loser is *recorded*, not deleted — negative results are deliverables.
3. **Honest negatives.** If your change loses, regresses, or only partially
   works, say so in the PR. This project's history includes a design round
   that lost to Python and says so in DESIGN.md; keep that standard.
4. **The grammars move together.** `tools/tokens/grammar.peg`,
   `tools/grammar/curt.lark`, and the generated `curt.gbnf` must stay
   membership-equivalent — the Rust parser is the oracle, and the divergence
   gates (`validate.py` in both tool dirs) enforce it. Never hand-edit
   `curt.gbnf`; regenerate via `lark2gbnf.py`.
5. **Adjacency is semantic** (SPEC §1). Tooling must preserve token
   gluedness — a whitespace normalizer that ignores it changes program
   meaning.

## The gate

One command runs everything CI runs — they are literally the same script:

```sh
ci/check.sh
```

cargo tests (126+), clippy `-D warnings`, both grammar divergence gates,
GBNF determinism, and the token cost table. A PR that weakens a gate or
masks an exit code will not merge.

## Commits

On `main`, imperative subject, body explains the *measured why*. Keep
corpus programs as literal files — never re-type them through shell
escaping (we learned this the hard way; see `tools/tokens/count.py`).

## Code style

Match the surrounding code. Identifiers in curt programs are
semantic-but-single-token (`buf`, `idx`, `acc`) — never single-letter golf:
measured accuracy cost, zero token gain.
