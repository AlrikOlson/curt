# curt and Zerolang — comparison note v0 (recon)

*Recorded 2026-06-12, from primary sources and local measurement. This is
the opening artifact of a pre-registered comparison; the measured verdict
comes from the head-to-head evaluation defined below, not from this note.
Every number here reproduces via the cited commands.*

## The two bets

Vercel Labs released **Zerolang** on 2026-05-17
([github.com/vercel-labs/zerolang](https://github.com/vercel-labs/zerolang),
Apache-2.0, "The Programming Language for Agents"; 5,001 GitHub stars in its
first 28 days). Zerolang and curt are rival answers to the same question —
*how should AI agents author programs cheaply and reliably?*

- **curt** bets on **minimal text**: the LLM tokenizer is treated as the
  ISA; every construct's o200k cost is measured before admission; agents
  emit ordinary source text, optionally under a grammar mask that makes
  syntax errors impossible; single-line JSON diagnostics with verbatim
  `fix` fields drive repair.
- **Zerolang** bets on **no text authorship at all**: `zero.graph` is the
  program database; agents `zero query` and `zero patch` it with checked,
  hash-guarded edits; `.0` text files are projections for human review.
  The compiler speaks JSON (`--json` on every command, `schemaVersion: 1`).

These bets are measurable against each other because patch commands are
themselves model output: tokens either way.

## Verified facts (2026-06-12, Zero v0.3.2)

- **Runnability:** Zero installs from its public script and runs locally.
  The full loop (`init` → `patch` → `run` → `export` → `import`) was
  executed in a sandbox. Their docs warn it is experimental; their
  contributor notes state breaking changes are deliberate policy, so all
  numbers below pin v0.3.2.
- **Diagnostics:** `zero check --json` emits structured diagnostics:
  `severity, code, message, path, line, column, length, expected, actual,
  help, fixSafety, repair{id, summary}, related`. In the captured case the
  `repair` field — the machine-actionable repair surface — resolved to
  `id: "manual-review"` ("Inspect the diagnostic fields and choose a repair
  manually"), i.e. no applicable fix. curt's diagnostic carries a `fix`
  string intended to be applied verbatim. Whether Zero's typed repair ids
  resolve to actionable repairs more often than this sample, and at what
  token cost, is the subject of the diagnostics tournament (saga stage 3).
- **Token accounting:** Zero's `zero tokens` command counts *lexer* tokens
  (kind/text/position), not LLM-tokenizer tokens. Measured LLM-token cost
  as a design gate remains, to our knowledge, unique to curt among the two.

## First numbers (o200k_base; toy scale — indicative only)

| artifact | o200k tokens |
|---|---|
| one diagnostic: curt (single-line JSON + verbatim fix) | **41** |
| one diagnostic: Zero (`--json`, captured TYP001) | 114 |
| one diagnostic: Zero (human 5-line form) | 61 |
| hello world: curt (`print "hello from zero"`) | **6** |
| hello world: Zero `.0` projection | 23 |
| hello world: Zero patch-command route | 26 |

Caveats, stated plainly: these are single toy artifacts, not a benchmark.
Zero's patch route should amortize on *edits to large programs* (patch a
node vs re-emit a file) — that is its actual bet, and the loop-level
evaluation below is designed to test exactly that. Zero's explicit
capability types (`world: World`, `raises`) cost tokens by design; that is
a safety tradeoff, not an oversight. Reference-file comparisons on shared
RosettaCode tasks (e.g. Zero's `99-bottles-of-beer.0` at 114 tokens) are
algorithm-confounded — their references use different structures than ours
— and are therefore secondary evidence at best.

## curt under an external methodology

Martin Alderson's independent study ("Which programming languages are most
token-efficient?", 2026-01-08) tokenized RosettaCode solutions with the
GPT-4 tokenizer and averaged per task; its 19-language spread ran from
Clojure (109) to C (~2.6× Clojure), with the array language J at 70 on a
follow-up set. Replicating the method on 10 RosettaCode-canon tasks
expressible in curt v0.3 (every program executed and output-verified before
counting):

**curt: 37.6 cl100k tokens average (n=10).**
Reproduce: `.ci-venv/bin/python tools/bench/rosetta/measure.py`

The task sets differ (the post's exact list is unpublished; ours skews
small), so this is an indication of scale, not a leaderboard entry.

## Where each side stands today

| axis | curt | Zerolang |
|---|---|---|
| diagnostic token cost (captured samples) | 41 | 61–114 |
| diagnostic actionability (captured samples) | verbatim `fix` | `repair: manual-review` |
| LLM-token cost as a design gate | yes (o200k-measured admission) | no (lexer-token accounting) |
| constrained-decoding artifacts | shipped, 0/200 violations measured | none found in repo surface |
| authoring surface | text (6-token hello) | graph patches (26-token hello) |
| edit amortization on large programs | unmeasured | unmeasured (their bet) |
| backing | independent project | Vercel Labs, 5k stars/month |
| maturity warnings | v0.3.1, research project | "expect breaking changes, security issues" |

## The pre-registered head-to-head

Frozen before any lane is generated (provenance: pinned reasoning step 126,
2026-06-12): nine shared RosettaCode tasks (100-doors, 99-bottles-of-beer,
fizzbuzz, fibonacci-sequence, factorial, greatest-common-divisor,
reverse-a-string, a-b, sum-and-product-of-an-array — all present in Zero's
own `benchmarks/rosetta` corpus); each language generates from the same
behavior-level prompt with its own canonical agent documentation (curt's
cheat sheet; Zero's version-matched `zero skills` output); single shot plus
up to two repair turns with native-format diagnostics fed back verbatim;
Python control lane; two models, three samples per cell; all lanes frozen
before grading. Verdict criterion, fixed in advance: curt must win at least
two of (a) loop dollars per solved task at success parity (±5pp),
(b) median o200k output tokens on solved cells, (c) repair convergence
turns. Results will be published whichever way they fall; a loss on any
axis becomes a measured gap list.

*Convergent context: independent 2026 essays arrived at curt's design
doctrine — "an LLM-native language would essentially be a very rich IR
that no human would want to write, paired with a human-facing projection"
(akitaonrails.com, 2026-02-09; compare DESIGN.md: human readability is a
derived view, not a source property) — and Armin Ronacher's "A Language
For Agents" (lucumr.pocoo.org, 2026-02-09) argues the category is viable.
Zerolang independently chose the same projection framing from the graph
side. The category is real; the measurements will decide the rest.*
