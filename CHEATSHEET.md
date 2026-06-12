# curt cheat sheet

curt is a general-purpose language optimized for LLM output tokens. Whole
program = toplevel statements, run in order. No imports, no `main`, no
significant indentation. Comments `#` (never emit them). Statements end at
newline or `;`.

## Core shape

```
greet name = "hi {name}"        # equation: name params = expr
total = [1,2,3].sum             # binding (= rebinds; x += 1 works)
(a, b) = (1, 2)                 # tuple destructuring
print greet "ana"               # application is juxtaposition: f x y
```

- Equation body: expression, or `{ block }` (value = last expression).
- `ret e` returns early from the enclosing equation.
- Strings interpolate: `"{x} and {y.len}"`. Escapes `\n \" \\ \{`.
- Names lowercase (`buf`, `idx`, `acc`); types Capitalized. Prefer short
  semantic words, never single letters.

## Application, pipes, projections

- `f x y` calls f with x and y. No parens, no commas. Parenthesize
  sub-calls: `print (show 2.5)` — though extra args re-nest rightward
  automatically, so `print show 2.5` also works.
- UFCS: `x.f a` ≡ `f x a`. So `xs.len`, `s.upper`, `xs.map g`.
- Pipeline `|` feeds the value as the LAST argument of the next stage:
  `xs | keep .active | top 2 .score | map .name`
- Bare `.field` is a lambda: `top 2 .score` ≡ `top 2 (x -> x.score)`.
- Lambdas: `x -> e`, `acc x -> e`. A lambda body stops at `|`, so bare
  lambda stages compose: `xs | map x -> x * x | sum`.
- The pipe takes the WHOLE expression on its left:
  `row.split "," | sum` pipes the split result.
- `print` and `?` wrap a pipeline, never head it: `print (xs | sum)`,
  `print (m["k"] ? 0)` — `print xs | sum` pipes print's unit (checker
  rejects it). Same for operators: `print (total / 3)`, NEVER
  `print total / 3` (that divides print's unit).
- Dots chain only when GLUED: `xs.keep(f).map(g)` chains; with spaces,
  chain with `|` instead.
- Application binds tighter than operators: `c.write x + y` is
  `(c.write x) + y`.
- An `if`/`match` expression can't sit bare as a call argument — bind it
  first (`word = if ... ; print word`) or parenthesize.

## Adjacency is meaning (the three traps)

- `x.f` (glued) = field access; `x .f` (spaced) = passing projection `.f`.
- `x?` (glued) = propagate error; `a ? b` (spaced) = rescue (if `a` is err
  or missing, yield `b`).
- `Pt{x:1}` / `f(x)` / `xs[0]` glued = record literal / call / index;
  spaced they become juxtaposition arguments. Keep them glued.

## Types

Inference is total — annotate nothing (except `pub`/FFI). Primitives `int
float str bool bytes`. Lists `[1,2]`, tuples `(1, "a")` (fields `.0 .1`),
records. `xs + ys` concatenates lists; `acc += [x]` appends. A mixed
literal like `[7, "ok"]` is a `[int | str]` union list. Multiline list
literals are fine (newlines inside `[ ]` and `( )` are plain whitespace).
Map literal: `m = {"k": 1, "n": 2}` (string keys, multi-line fine;
`{}` is the empty map; build dynamic keys with `m[k] = v`); index
`m["k"]` or `m.k`
(field syntax does key lookup), rescue a missing key: `m["k"] ? 0`;
iterate `m.pairs` which yields records with fields `.k` and `.v`.
Maps also arrive from `counts`, `.json`, `group`. Records (ident
keys, NOT indexable — distinct from maps):

```
type Pt = {x float, y float}
p = Pt{x:3, y:4}
print p.x
```

Unions are untagged: `int | str`, `T | err`. `match` narrows by runtime
type — prims and declared type names (structural: `Pt q` matches any
value fitting Pt's shape) — and is exhaustiveness-checked over union
members:

```
show v = match v { float x -> "num {x}", str s -> "sym {s}" }
```

Arms: type patterns (`int n`), literals (`"("`, `42`), tuples, `_`, bare
name (binds anything). Mixed int/float math joins to float (`1 + 2.5` is
`3.5`); `/` on two ints is integer division; convert with `.float` `.int`.
Optional: annotate bindings `x: int = 0`; declare fn contracts with
`f :: int int -> int` — never required, enforced when present.

## Control flow (all expressions)

```
if c { a } else { b }           # else optional; else if chains
while c { ... }
for x in xs { ... }             # lists, maps (pairs), strings (chars)
for i in range 10 { ... }       # 0..9; range 1 16 = 1..15
                                # no continue/break — restructure the loop
match v { ... }
go f x                          # spawn lightweight thread
```

Inside `for/while/if/match` headers, `{` always starts the block — don't
put a record literal in a header unparenthesized.

## Errors

Failable ops return `T | err`. Two one-token tools:

```
cfg = fs.read "app.cfg" ? ""    # rescue: fallback if the READ errs
data = (fs.read p)?             # propagate: return err to caller
ok = match v { err e -> "failed: {e}", int n -> "got {n}" }
```

`err e` is a match pattern binding the message. Diagnostics are
single-line JSON with a `fix` field — apply it verbatim.

## Stdlib verbs (all 1 token; `x.f a` ≡ `f x a`)

- seq/str: `len map keep fold sum min max sort rev top group counts pairs
  first last flat join split words lines chars bytes trim lower upper
  replace range`
- conv: `int float str json`
- io (capability-gated): `print args fs.read fs.write net.listen c.lines
  c.write`
- `keep` = filter. `top n .f` = sort desc by `.f`, take n. `group .f` →
  list of `{k, v}`. `counts` = frequency map. `fold init acc x -> e`.
- There is NO `contains` (membership is `x in s` — strings and lists), NO
  key function on `sort`/`min`/`max` (use `top n (x -> key)`), NO
  `split ""` for characters (use `.chars`).

## Worked examples

```
for g in sales.group .city { print "{g.k} {g.v | map .amt | sum}" }

pub dist :: Pt Pt -> float       # :: only on exports/FFI
dist a b = ((a.x-b.x)**2 + (a.y-b.y)**2).sqrt
```

## Dense beats loops (pairs verified identical — write the dense form)

```
best = xs[0]                                # 22 tok
for x in xs { if x > best { best = x } }
best = xs.max                               # 4 tok

out = ""; sep = ""                          # 27 tok
for w in ws { out = out + sep + w; sep = "-" }
print (ws | join "-")                       # 7 tok

best = us[0]                                # 27 tok
for u in us { if u.score > best.score { best = u } }
print (us | top 1 .score | map .name | join "")   # 16 tok
```

Loop only when state threads between iterations; otherwise verbs first.

## Don't write Python

No `def lambda import return elif try/except f"" [x for x in]`. No `:` 
after headers — braces. No `len(x)` — `x.len`. Slices `xs[1:]` exist.
`and or not` (not `&& || !`). `true false`. Strings are double-quoted;
`'...'` is a raw string (no interpolation — handy inside `"{...}"` holes).
`"{}"` is literal braces. The parser forgives common slips (`return`,
`elif`, `f(x, y)`, `True`, `++`) but emit canonical forms.
