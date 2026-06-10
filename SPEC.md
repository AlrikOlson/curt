# cmm language specification v0.1

**Status:** 2026-06-10 · implements [DESIGN.md](DESIGN.md) v0.2 · grammar
machine-validated against [corpus/](corpus/) (20/20, `tools/tokens/validate.py`)
· all token numbers measured by [tools/tokens/count.py](tools/tokens/count.py)
(o200k_base; Anthropic count-tokens hook when `ANTHROPIC_API_KEY` is set).

This document is sufficient to implement an interpreter/compiler from. Where a
decision was settled by measurement, the tournament record (§12) names the
losing candidates. DESIGN.md v0.2's example numbers were draft-round
measurements; the canonical numbers are §11 of this spec.

## 1. Lexical structure

- **Encoding:** UTF-8 source; the grammar itself is ASCII-only. String
  literals may contain any UTF-8.
- **Names:** `[a-z_][A-Za-z0-9_]*` (values/functions), `[A-Z][A-Za-z0-9_]*`
  (types). Canonical style: semantic single-token words (`buf`, `idx`, `acc`).
  The compiler **lints any identifier that tokenizes above 1 token** under the
  reference tokenizer (warning, not error).
- **Keywords** (all measured 1 token): `if else while for in match type ret
  go pub and or not`. `true false` are literals.
- **Numbers:** `123`, `1.5`, optional width suffix `7u64` / `42i32`
  (suffixes `i8|i16|i32|i64|u8|u16|u32|u64`; each 2 tokens, used only in
  low-level code).
- **Strings:** `"..."` with escapes (`\n`, `\"`, `\\`) and **interpolation**:
  `{expr}` inside a string splices the expression (any value; formatted as by
  `print`). Literal brace: `\{`.
- **Comments:** `#` to end of line. Agents are instructed never to emit them;
  they exist for humans and generated views.
- **Statement separation:** newline or `;`. **Indentation is never
  significant** (measured: `"\n    "` = 2 tokens vs `"\n"` = 1; and models
  corrupt indentation in surgical edits). The canonical formatter emits
  2-space indents for human viewing; the grammar ignores them.
- **Adjacency is semantic** *(amended in interp-b after the machine-validated
  grammar surfaced it)*: whether two tokens touch changes meaning in exactly
  three construct families — `x.f` (field access, glued) vs `x .f` (spaced:
  `.f` is a projection-lambda argument); `x?` (glued: propagate) vs `x ? y`
  (spaced: rescue); and `Pt{…}` / `f(…)` / `xs[…]` (glued: record literal /
  call sugar / index-slice) vs their spaced forms (juxtaposition arguments).
  Tooling MUST preserve gluedness in these families — a whitespace
  normalizer that ignores adjacency changes program meaning. `cmm fmt`
  therefore preserves input adjacency by default and normalizes only
  spellings, separators, and layout.
- **No imports.** Whole-program compilation; the stdlib and host capabilities
  (§9) are ambient. No visibility keywords except `pub` (§8). No `main` —
  toplevel statements execute in order.

## 2. Grammar

Normative grammar: [`tools/tokens/grammar.peg`](tools/tokens/grammar.peg)
(parsimonious PEG; the validation gate parses the entire corpus). Summary of
the structural rules an implementer must honor:

1. **Statements:** `type` declarations, `pub`/plain `::` signatures,
   equations, bindings, destructuring binds, compound assignment, `for`,
   `while`, `ret`, `go`, expression statements.
2. **Equations:** `name p1 p2 = body` where body is an expression, a block,
   or a single `for`/`while`/`go` statement (unit-valued). One or more
   parameters distinguishes an equation from a binding (PEG ordered choice).
3. **Application is juxtaposition**, flat and left-associated: `f x y` is one
   application node `(f, [x, y])`; **arity and grouping are resolved by type
   inference, not the grammar** (so `print show 2.5` parses flat and
   elaborates to `print (show 2.5)`).
4. **Header brace rule (Go-style):** inside `for`/`while`/`if`/`match`
   headers, `{` always begins the block — brace-initial atoms (anonymous
   records, blocks) are excluded from header expressions. Parenthesize a
   record literal if one is ever needed in a header.
5. **`?` disambiguation by adjacency:** postfix `x?` (no space) propagates an
   error; spaced `a ? b` is binary rescue. (§7.)
6. **Precedence**, loosest → tightest: rescue `?` · pipeline `|` · `or` ·
   `and` · `not` · comparisons (`== != < <= > >= in`) · `+ -` ·
   `* / % ^` · `**` · unary `-` · application (juxtaposition) · postfix
   (`.name`, `.N`, `[...]`, propagate-`?`) · atom. (`^` is bitwise xor; it
   sits with `*` in v0.1 — low-level code parenthesizes, as the corpus does.)
7. **Blocks are expressions:** value = last expression; statements inside
   separated by newline/`;`. `(v, r) = expr ts[1:]; (v, r[1:])` is a valid
   one-line block body.
8. **Lambdas:** `x -> e`, `acc x -> e` (multi-param). A lambda is permitted
   wherever an application argument is (tried before a bare name by ordered
   choice).
9. **Projection atoms:** bare `.name` / `.N` is a one-token-plus lambda:
   `top 2 .score` ≡ `top 2 (x -> x.score)`.

## 3. Types and inference

- **Primitives:** `int` (i64), `float` (f64), `str`, `bool`, `bytes`; sized
  `i8…u64` for low-level work. `unit` is the value of statements (never
  written).
- **Composites:** lists `[T]`, maps `{K: V}` (string-keyed maps are the
  literal default), tuples `(A, B, …)`, records (nominal via `type Name =
  {field T, …}`, or structural when anonymous), functions.
- **Untagged unions:** `A | B` — the measured answer to the ADT tax
  (DESIGN.md §1). No constructors; values carry their runtime type tag;
  `match` narrows (§5). A union is well-formed only over distinguishable
  runtime types.
- **Inference is total for non-exported code:** Hindley–Milner-style over the
  flat application form; the elaborator simultaneously resolves application
  grouping (§2.3) and types. Annotations (`x: int = …`, 1 extra token each)
  are permitted anywhere, **required nowhere except** exported/FFI
  signatures (§8). If inference is ambiguous, the diagnostic names the
  smallest expression needing one annotation (§7 error shape).
- **Generics:** inferred polymorphism only in v0.1; there is no surface
  syntax for type parameters (a signature names concrete types or unions).
  Deferred (§13).
- **Numerics:** `int` and `float` do not implicitly mix; `/` on `int` is
  integer division (corpus 04, 14); `.float` / `.int` convert. Sized unsigned
  types wrap on overflow (corpus 16, FNV-1a); signed overflow traps.

## 4. Bindings, mutation, equality

- `x = e` binds or rebinds (shadowing within a block is rebinding; capture in
  closures is by reference to the binding). `x += e` etc. compound-assign.
- `(a, b) = e` destructures tuples; works in `for` patterns.
- **Value semantics at boundaries:** lists/maps/records assign and pass by
  reference (RC, §6) but the stdlib is persistent-style where cheap; v0.1
  keeps it simple — aliasing is observable, as in Python. (Deferred:
  copy-on-write experiments, §13.)
- **`==` is structural equality** on all values (tournament: tie with `=`,
  decided by ambiguity-removal + Python/C muscle memory). `=` in expression
  position is Postel-accepted as `==` (§10).

## 5. Control flow

- `if c { a } else { b }` is an expression; `else` optional (unit when
  absent); `else if` chains.
- `while c { … }`, `for pat in e { … }` — iterate lists, maps (pairs),
  strings (chars), streams (`net.listen`, `c.lines`), `range n` /
  `range a b` (tournament: `range` beat `..` on cost and removed an operator).
- `ret e` returns early from the enclosing equation (last-expression remains
  the canonical return; `ret` exists for guard-style exits — corpus 04).
- `go e` spawns `e` on a lightweight thread; `unit`-valued. Channels/select:
  deferred (§13); v0.1 concurrency is spawn + the host's blocking streams.
- `match v { pat -> e, … }`: first matching arm; patterns are type-narrowing
  (`float x`, `str s` — binds the narrowed value), literals (`"("`, `42`),
  tuples, `_`, or a bare name (binds anything). **Exhaustiveness over a
  union's members is checked**; non-exhaustive match on a union is a compile
  error with a fix-suggesting diagnostic.

## 6. Memory model — reference counting (decided, with measurements)

**Decision: RC + cycle collector.** Deterministic destruction, no GC pauses,
zero source-token cost.

The alternative (Rust-style ownership, even inferred) was rejected on
measurement: the 6 Rust corpus files contain **53 ownership-ceremony
occurrences** (25 borrows, 10 `mut`, 9 unwrap/ok, 2 clones, 3 derefs, 3
casts, 1 `move`) — **≥5% of all Rust corpus tokens** (lower bound; `.unwrap()`
alone is 4 tokens per use). Ownership *inference* would hide annotations but
still surfaces clone/borrow decisions in diagnostics the model must then
resolve — token cost moves to the repair loop. RC's runtime overhead is the
accepted price; NanoLang (the LLM-targeted language with a proved core)
independently chose RC + cycle collector. Escape hatch for hot paths: deferred
arena/region annotations (§13), never required.

## 7. Errors

- Failable operations return `T | err` (an `err` value carries code, message,
  and origin). The union composes with §3.
- **Postfix `x?`** (touching, 1 token): if `err`, return it from the
  enclosing equation; else unwrap. **Spaced `a ? b`** (1 token): if `a` is
  `err` (or a missing-key/None-like absence), evaluate and yield `b`.
  Measured in v0.1 drafts at 4 tokens vs Python's 24-token try/except-retry
  shape.
- **Diagnostics are prompts** (machine-first): single-line JSON —
  `{err:"type_mismatch", at:"19:7", want:"float", got:"str", fix:"insert .float after w"}`
  — designed to be fed back verbatim for one-edit self-repair. Every
  diagnostic category ships a golden fixture in interp-mvp.

## 8. Exports and FFI

- `pub name :: T1 T2 -> R` followed by the equation exports it (tournament:
  `pub` beat the bare-`::`-marks-export rule, 8 vs 14 tokens; `::` lines are
  the *type declaration*, mandatory only here because FFI cannot be
  inferred). Non-exported code needs neither.
- The same `::` form declares external C/wasm imports (host supplies the
  symbol). The capability model is host-granted: `fs`, `net`, `env`, and any
  registered tool namespaces exist only if the embedding grants them
  (sandbox-first; see DESIGN.md §6).

## 9. Stdlib (v0.1 corpus surface)

Admission rule: a verb enters only if it saves more corpus tokens than its
cheat-sheet line costs (DESIGN.md P4). The v0.1 set (all measured 1 token,
shown with receiver style; UFCS: `x.f a` ≡ `f x a`):

- **seq/str:** `len map keep fold sum min max sort rev top group counts pairs
  first last flat join split words lines chars bytes trim lower upper
  replace range`
- **conv:** `int float str json`
- **io (capability-gated):** `print args fs.read fs.write net.listen
  c.lines c.write`
- `top n .f` = sort-desc by projection, take n. `group .f` = list of
  `{k, v}` pairs. `counts` = frequency map. (Corpus 08/10/18 exercise these.)

## 10. Postel set (accepted slips → canonical)

The parser accepts, and `fmt` canonicalizes (never errors): `==`←`=` (in
expression position), `&&`→`and`, `||`→`or`, `!x`→`not x` (expression
position), `True/False/None`→`true/false/()`, `return`→`ret`, `elif`→`else
if`, trailing commas, `f(x, y)`→`f x y` (paren-call form), smart quotes.
Rationale: language-confusion drift toward Python/Rust is measured behavior
(arXiv 2503.13620); recoverable habits must never cost a repair loop.

## 11. Canonical cost table (the CI gate)

`tools/tokens/count.py` over `corpus/` — o200k_base, 2026-06-10:

| snippet | cmm | Py | Go | Rust |
|---|---|---|---|---|
| 01 hello | 6 | 6 (1.00×) | — | — |
| 02 hyp | 20 | 27 (1.35×) | — | — |
| 03 fib | 32 | 29 (0.91×) | — | — |
| 04 binsearch | 98 | 98 (1.00×) | 136 (1.39×) | 147 (1.50×) |
| 05 records | 52 | 66 (1.27×) | — | — |
| 06 union+match | 35 | 38 (1.09×) | — | — |
| 07 errors | 29 | 40 (1.38×) | 141 (4.86×) | 100 (3.45×) |
| 08 pipeline | 55 | 80 (1.45×) | — | — |
| 09 strings | 18 | 24 (1.33×) | — | — |
| 10 group | 53 | 67 (1.26×) | 92 (1.74×) | 102 (1.92×) |
| 11 filelines | 24 | 22 (0.92×) | — | — |
| 12 fold | 20 | 29 (1.45×) | — | — |
| 13 tuples | 36 | 34 (0.94×) | — | — |
| 14 while-acc | 60 | 56 (0.93×) | — | — |
| 15 spawn | 20 | 35 (1.75×) | — | — |
| 16 bitops | 50 | 56 (1.12×) | — | — |
| 17 export/ffi | 16 | 11 (0.69×) | — | — |
| 18 wordfreq | 31 | 39 (1.26×) | 155 (5.00×) | 157 (5.06×) |
| 19 parser | 257 | 234 (0.91×) | 416 (1.62×) | 439 (1.71×) |
| 20 server | 31 | 55 (1.77×) | 94 (3.03×) | 123 (3.97×) |

**Medians: 1.19× vs Python (n=20) · 2.38× vs Go (n=6) · 2.69× vs Rust (n=6;
compiled subsets are small-n and flagged as such).** Honest distribution: cmm
beats Python on 13/20, ties 2, loses 5 — the losses are pure-algorithm
snippets (0.91–0.94×) and `17_export` (0.69×: Python has no export ceremony
at all). This is consistent with DESIGN.md's parity finding and sits inside
the token-bench targets (parity-to-1.2× vs Python; ≥1.8× vs Go).

Per-construct prices: `count.py --constructs` (34 constructs; the regression
gate fails any grammar/stdlib change that worsens the corpus).

## 12. Tournament record (decisions + losers)

| decision | winner | loser(s) | decided by |
|---|---|---|---|
| equality | `==` | `=` | tie → ambiguity-removal, Py/C muscle memory |
| boolean ops | `and or not` | `& \| !` | tie → frees symbols for bitwise; Py-aligned |
| print verb | `print` | `say`, `out` | tie → maximal muscle memory |
| float name | `float` | `flt` | **cost** (flt = 2 tokens — abbreviation was *more* expensive) |
| arrow | `->` | `=>` | tie → one arrow everywhere |
| ranges | `range` fn | `..` operator | **cost** (wins 0-case by 1; −1 grammar operator) |
| propagate | postfix `?` | `try` | tie → Rust muscle memory; doubles as rescue |
| export | `pub` | `::`-line-as-export | **cost** (8 vs 14) |
| narrowing | `match` only | equation-head patterns | grammar simplicity (deferred, §13) |

## 13. Deferred to v0.2+ (recorded, not designed)

Zero-argument call syntax under juxtaposition; channels/`select`;
copy-on-write experiments at mutation boundaries; surface generics; arena
annotations for hot paths; equation-head patterns; `^` precedence tier;
trait/interface story; string `bytes` zero-copy views.

---
*Lineage: DESIGN.md v0.2 (direction, user-approved) → this spec (canonical).
Gates: `validate.py` 20/20 · `count.py` reproduces §11 exactly. Numbers
measured 2026-06-10 with tiktoken o200k_base.*
