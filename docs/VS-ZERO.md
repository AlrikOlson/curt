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
| diagnostic token cost (captured samples) | ~44 (typed, post-tournament) | 61–114 |
| diagnostic actionability (captured samples) | typed `repair{id,summary}` + `want`/`got` (tournament-adopted) | `repair: manual-review` |
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

## The diagnostics tournament (saga stage 3, 2026-06-12)

Zero's bet: stable error codes with **typed repair identifiers**. curt's
shipped design: a prose `fix` hint (in practice a canned per-error-class
string). The two designs were tournamented on curt's own repair corpus —
32 toolchain-verified broken programs (stratified over 26 distinct error
shapes), each rendered four ways and fed to claude-haiku-4-5 for repair
(1 sample/cell, ≤2 turns, verified-fix stdout as oracle, transcripts
frozen in `tools/bench/tourney/`; $0.54 total). The verdict rule was
pre-registered before any API call.

| rendering | diag o200k | turn-1 repair | final | lane $ |
|---|---|---|---|---|
| A — shipped (prose hint) | 38.4 | 18/32 | 21/32 | $0.152 |
| B — typed (Zero-style steelman) | 43.3 | 21/32 | 25/32 | $0.143 |
| C — hybrid (typed + hint) | 60.3 | 22/32 | 26/32 | $0.139 |
| D — typed + replacement payload | 82.8 | **32/32** | 32/32 | $0.107 |

**Verdict: ADOPT — Zero's design direction wins on curt's own corpus,
and curt shipped it the same day.** Typed fields beat the prose hint by
+9.4pp turn-1 success at 1.13× diagnostic tokens (the pre-registered
adopt gate was >5pp at ≤1.15×); the hybrid's extra prose added one cell
at 1.57× cost and failed its gate. curt's diagnostics now emit
`want`/`got`/`symbol`/`callee` typed fields plus a stable
`repair{id,summary}` operation (diag.rs, SPEC §7) — at single-line curt
economy (~44 tokens vs the 114 of Zero's captured multi-field form).
Two honest qualifiers: the margin is 3 cells at n=32, single-sample,
one model; and the adopted shape keeps SPEC §7's original `want`/`got`
vocabulary rather than arm B's `expected`/`actual` keys (the measured
treatment was typed-vs-prose, not key spelling).

**The headline is arm D.** When the diagnostic carries a rustc-style
machine-applicable replacement (`repair.replacement: [{line, new}]`),
repair is **32/32 single-shot — and the cheapest lane**, because turn
count dominates diagnostic size in loop dollars. D's payloads were
derived from the verified fixes (an oracle-assisted upper bound, not
shipped capability), so the number bounds the prize: a compiler that
synthesizes actual replacements converts ~34pp of residual repair
failure into first-turn success. That work is filed on the roadmap
(`fix-synthesis`). Neither Zero's `repair.id: "manual-review"` (its
captured behavior) nor curt's pre-tournament hints come close to this
bound.

Found while building the tournament, and recorded in the same spirit:
curt's *runtime* error path emits invalid JSON when the message embeds
quotes (nested-diagnostic case, `main.rs`; filed as `diag-esc-runtime`)
— a diagnostic that breaks parsers forfeits any arms race, ours
included. Prior-art note: we found no published measurement of typed
vs prose diagnostic feedback for LLM repair (searched 2026-06-12); the
frozen lanes above appear to be the first.

## The head-to-head (saga stage 4, 2026-06-12)

The pre-registered showdown ran exactly as frozen (pinned reasoning
step 126): the nine shared RosettaCode tasks, each language carrying its
own canonical teaching doc, single shot plus up to two repair turns with
native-format failure surfaces fed back verbatim, Python control, two
models × three samples per cell, temperature 1.0, prompt caching
requested identically everywhere. 162 cells, $0.77, all transcripts
frozen in `tools/bench/h2h/`. One pre-lane protocol note: Zero's
teaching doc is `zero skills get language` **plus** `stdlib` — the
language skill alone omits `std.fmt.i32`, and a smoke probe (discarded)
showed the model dying in three turns guessing conversion helpers; the
union matches the cheatsheet's syntax+stdlib coverage. The measured
consequence is itself a result: **Zero's canonical authoring docs cost
14,923 o200k tokens to curt's 2,018 — a 7.4× documentation tax.**

| model | lang | solved | $/solved | median out-tok (solved) | turns |
|---|---|---|---|---|---|
| haiku | curt | 21/27 | $0.0062 | **33** | 1.24 |
| haiku | zero | 24/27 | $0.0050 | 168 | 1.00 |
| haiku | python | **27/27** | **$0.0005** | 37 | 1.00 |
| sonnet | curt | **27/27** | $0.0020 | **35** | 1.00 |
| sonnet | zero | **27/27** | $0.0156 | 168 | 1.15 |
| sonnet | python | **27/27** | $0.0011 | 40 | 1.00 |

**Mechanical verdict against the pre-registered 2-of-3 condition: a
split decision, reported in full.**

- **sonnet: curt wins 3/3** — loop dollars at perfect success parity
  ($0.0020 vs $0.0156, 7.7× cheaper), median output tokens (35 vs 168,
  4.8× leaner), convergence turns (1.00 vs 1.15).
- **haiku: curt does not win (1/3)** — success parity fails (78% vs
  89%), which voids the dollar axis by rule. curt's six failures are
  two tasks at 0/3 each: 100-doors (the model repeatedly reached for a
  three-argument stepped `range` curt does not have) and 99-bottles
  (exact-format lyrics); the other seven tasks went 9/9.
- **pooled across models: curt does not win (1/3)** — parity misses by
  0.6pp (88.9% vs 94.4%). The pre-registration never specified
  cross-model aggregation; that is a pre-registration defect, recorded
  here rather than resolved post hoc in either side's favor.

Findings the matrix forces, whichever side one roots for:

1. **The output-token bet is decided: curt is ~5× leaner than Zero on
   the wire** (33–35 vs 168 median tokens on solved cells) — Zero's
   `pub fn main(world: World) -> Void raises` ceremony and per-line
   `check world.out.write(...)` calls are priced on every generation.
   Zero's own a-b reference is 46 o200k tokens to curt's 12.
2. **The documentation tax decides the dollar axis as much as the
   language does.** Frozen cache fields: haiku+curt read 109.5k input
   tokens at full price (its 2k sheet sits under the 4,096-token cache
   floor) while haiku+zero served 618.8k of its 15k doc as 0.1× cache
   reads. Zero's docs cache; curt's don't — yet curt still wins
   $/solved wherever success parity holds, and Zero's sonnet lane paid
   $0.42 for doc re-reads even cached.
3. **Python won the control on every cost axis** (54/54, $0.0008 per
   solved task). On nine small well-trained tasks, neither agent
   language beats the language the models grew up on. The agent-language
   category's case must rest on harder ground: diagnostics-driven
   repair, capability safety, and edit-loop economics on larger
   programs.

The saga verdict (stage 5) weighs this split; nothing here is restated
in curt's favor. Raw lanes, tasks, oracles, and both teaching docs are
committed and reproduce via
`.ci-venv/bin/python tools/bench/headtohead.py report`.
