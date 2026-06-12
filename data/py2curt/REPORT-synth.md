# corpus-synth pipeline report

Toolchain-verified synthesis: deterministic instruction templates
(we author the task; the model writes only the program), filtered by
check -> double-run determinism -> nonempty output -> feature
presence -> canonical-form dedup. No Python oracle exists for these
pairs; determinism, the checker, and a manual audit stand in.

Generator: claude-sonnet-4-6. Estimated API cost: $3.63 (input 89,805 + cached 4,957,680 + output 125,079 tokens).

| feature | accepted | rejections |
|---|---|---|
| maplit | 140 | check 104, duplicate 2 |
| match_err | 140 | check 91, duplicate 30 |
| rescue | 140 | check 24, duplicate 2 |
| rawstr | 89 | duplicate 311 |
| fs | 140 | duplicate 79, check 76, empty-output 16 |
| pipeline | 140 | feature-absent 90, check 7, duplicate 3 |
| numjoin | 26 | check 374 |
| annot | 140 | duplicate 5, check 3 |

Total: 955 pairs (94 held out as eval — every 10th accepted pair per feature).

## Correction: `rawstr` regenerated

The first run's fmt canonicalization erased raw `'...'`
strings (rewritten to escaped form) AFTER the feature-presence
check — the family was regenerated with canonicalization
skipped. New count: 81 (rejections this round: 319; est cost $0.48).

## Manual audit (50-pair random sample, seed 42)

Read in full. Findings:

- **No semantic mismatches**: every sampled program faithfully implements
  its instruction; spot-checked pipelines, rescues, match arms, and fs
  parsing are correct.
- **One systematic defect found and fixed**: the original run's fmt
  canonicalization rewrote raw `'...'` strings to escaped form AFTER the
  feature-presence check — all 89 original "rawstr" pairs contained no
  raw string. The family was regenerated with canonicalization skipped
  (see Correction above); all 81 replacement pairs verified to contain
  raw strings.
- **Instruction diversity is narrow in 2-3 families** (match_err and
  maplit samples repeat structurally, varying mainly in literals); fs
  shows genuine structural diversity across samples. Dedup here is
  exact-canonical-hash, not literal-normalized — a literal-normalized
  dedup is specified for synth-v2.
- **numjoin under quota (26/140)**: 374 checker rejections, dominated by
  the precedence footgun (`print total / 3` parses as `(print total) / 3`).
  The checker catches it with the correct diagnostic; the model commits
  it persistently. This is simultaneously a cheatsheet teaching gap and
  prime repair-triple material for synth-v2.
