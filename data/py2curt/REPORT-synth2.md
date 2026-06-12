# synth-v2 pipeline report

Tiered verification: Tier A pairs match a template-computed expected
output (differential verification); Tier B pairs carry k-sample output
consensus (k=4, numeric-normalized clustering, strict majority).
Winners are the fewest-o200k-token members of their clusters.

- pairs: 1524 (407 Tier A / 1117 Tier B; 281 from two-feature + 103 from three-feature compositions; 152 held out)
- DPO preference pairs: 297
- repair triples: 733 (repair attempts that failed re-verification: 345)
- consensus failures (ambiguity signal): 308
- oracle mismatches (deterministic-but-wrong, the class v1 could not catch): 424
- estimated batch-rate cost: $8.61 (model claude-sonnet-4-6, k=4, temp 0.9)

## Tier-B audit (30 winners, 18 from compositions; 4 repair triples read)

- Composed pairs show genuine feature interaction (match+rescue+pipeline,
  fs aggregation into dynamically-built maps); no semantic mismatches
  with instructions found in the sample.
- Repair triples are ecologically valid: the dominant numjoin precedence
  footgun (`print x / y`) is fixed with parentheses; match-in-statement
  errors are restructured to expression form.
- Discovery: two repairs reveal that MULTI-LINE map literals fail to
  parse ("expected }, found Colon") — newlines are whitespace inside
  `(`/`[` but not `{`, and models naturally write maps multi-line.
  Filed as a language-polish item.
- Raw strings survive in v2 pairs (no fmt canonicalization step exists
  in this harness — the v1 erasure class is structurally absent).
