# py2curt real-source pipeline report

Real human-written Python (MBPP, HumanEval) adapted to stdout
programs and run through the same verification spine as the
generated corpus. High rejection is expected: the transpiler's
subset was defined by the seed generator, and wild Python uses
constructs outside it. Every rejection is tagged below.

Sources: MBPP (CC BY 4.0, google-research), HumanEval (MIT,
openai); `mbpp_plus` marks membership in the hand-verified
EvalPlus MBPP+ subset (Apache-2.0).

| source | verified pairs | problems | yield | split |
|---|---|---|---|---|
| mbpp | 225 | 974 | 23.1% | train |
| humaneval | 41 | 164 | 25.0% | eval (held out) |

## Rejection taxonomy

| tag | count |
|---|---|
| curt-check | 215 |
| unsupported:stmt | 200 |
| unsupported:builtin | 171 |
| no-printable-tests | 103 |
| unsupported:method | 34 |
| unsupported:expr-stmt | 33 |
| unsupported:binop | 20 |
| unsupported:comp-target | 15 |
| unsupported:const | 15 |
| curt-runtime | 12 |
| unsupported:for-shape | 11 |
| unsupported:comp-multi | 10 |
| oracle-error:NameError | 9 |
| unsupported:augassign-op | 3 |
| output-mismatch | 3 |
| unsupported:multi-assign | 3 |
| unsupported:slice-step | 3 |
| unsupported:cmp | 2 |
| unsupported:cmp-chain | 2 |
| unsupported:assign-target | 2 |
| unsupported:expr | 2 |
| unsupported:bare-return | 2 |
| unsupported:dict-key | 2 |

## Triage decisions (first-contact taxonomy)

Extensions accepted (semantics-exact, measured against this corpus):
single-param lambdas; `xs[::-1]` -> `.rev`; subscript aug-assign;
variadic `min`/`max` -> list verbs; `list(range(..))` and
`list(<comprehension>)` identities; unknown builtins now reject at
transpile time with an honest `builtin` tag (previously leaked to
the curt checker as `unknown_name`); the adapter lowercases
Capitalized function names (curt reserves them for types).

Skipped by design: imports/modules (`stmt: Import` — stdlib growth
is a separate measured-admission decision), dict/set literals,
`break`/`continue`, `None`, bitwise operators, tuple loop targets,
in-place mutation calls, multi-generator comprehensions, and the
checker's numeric strictness (`curt-check` residue).

## Held-out split

The split is by SOURCE: every HumanEval pair is `split: eval`
and must never be trained on; all MBPP pairs are `split: train`.
Splitting by source (not randomly) prevents near-duplicate
leakage across the train/eval boundary.

## Decontamination

Every pair is scanned against the frozen evaluation suites
(token-shingle Jaccard, normalized identifiers/numbers/strings):

```
.ci-venv/bin/python tools/py2curt/decontam.py \
  data/py2curt/pairs-real.jsonl.gz data/py2curt/pairs.jsonl.gz
```

The scan exits non-zero on any hit at Jaccard >= 0.5.
