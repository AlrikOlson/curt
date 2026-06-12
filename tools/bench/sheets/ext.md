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
type and is exhaustiveness-checked over union members:

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


## Worked examples (all execution-verified)

Exact-format text output — tuple destructuring, singular/plural, blank-line
separation between blocks but not after the last, interpolation everywhere:

```curt
items = [("widget", 3), ("gizmo", 1), ("bolt", 12)]
total = 0
for (name, n) in items {
  unit = if n == 1 { "unit" } else { "units" }
  print "{name}: {n} {unit}"
  total += n
}
print ""
print "TOTAL: {total} items in {items.len} lines"
```
prints `widget: 3 units` / `gizmo: 1 unit` / `bolt: 12 units` / blank /
`TOTAL: 16 items in 3 lines`.

Countdown with exact verse separation (blank line BETWEEN groups only):

```curt
n = 3
while n > 0 {
  plural = if n == 1 { "step" } else { "steps" }
  print "{n} {plural} remaining"
  print "tick"
  n -= 1
  if n > 0 { print "" }
}
print "done"
```

Stepped iteration — `range a b step` (start, stop-exclusive, step):

```curt
for i in range 1 11 { print (i * i) }
```

FizzBuzz:

```curt
for i in range 1 101 {
  s = if i % 15 == 0 { "FizzBuzz" } else if i % 5 == 0 { "Buzz" } else if i % 3 == 0 { "Fizz" } else { "{i}" }
  print s
}
```

Word frequency — pipelines, group, top:

```curt
for p in (fs.read args.1).lower.words.counts | top 10 .v { print "{p.k} {p.v}" }
```

Grouping records:

```curt
sales = [{city:"NY", amt:50}, {city:"LA", amt:30}, {city:"NY", amt:20}]
for g in sales.group .city { print "{g.k} {g.v | map .amt | sum}" }
```

Binary search — while, indexing, integer midpoint:

```curt
bs xs t = {
  lo = 0
  hi = xs.len - 1
  while lo <= hi {
    mid = (lo + hi) / 2
    if xs[mid] == t { ret mid }
    if xs[mid] < t { lo = mid + 1 } else { hi = mid - 1 }
  }
  -1
}
print bs [1,3,5,7,9,11] 7
```

Errors end-to-end — propagate `?`, rescue `a ? b`, match with err arm:

```curt
load p = (fs.read p).json
cfg = load "app.cfg" ? {}
print (cfg["port"] ? 8080)
```

Fold and reduce:

```curt
print [1,2,3,4].fold 0 acc x -> acc + x
```

Tuples and destructuring:

```curt
minmax xs = (xs.min, xs.max)
(lo, hi) = minmax [3,1,4,1,5]
print "{lo} {hi}"
```

String building and joins:

```curt
batches = [[3, 1], [4, 1, 5]]
acc = []
for b in batches { acc += b }
acc = acc + [9]
print (acc | map str | join " ")
tag v = match v { int n -> "i{n}", str s -> "s{s}" }
for v in [7, "ok"] { print tag v }
print (range 2 5 | map (n -> n * n) | sum)
```

Map literals and dynamic keys:

```curt
hash s = {
  h = 14695981039346656037u64
  for b in s.bytes { h = (h ^ b) * 1099511628211 }
  h
}
print hash "curt"
```

Concurrent server — go, capabilities:

```curt
handle c = for ln in c.lines { c.write (ln.upper + "\n") }
for c in net.listen 8080 { go handle c }
```

Reading files (capability-gated, run with --fs):

```curt
for ln in (fs.read "log.txt").lines | keep x -> "ERR" in x { print ln }
```


Records and field access:

```curt
type Pt = {x float, y float}
dist a b = ((a.x-b.x)**2 + (a.y-b.y)**2).sqrt
print dist Pt{x:0, y:0} Pt{x:3, y:4}
```

Union types with match:

```curt
show v = match v { float x -> "num {x}", str s -> "sym {s}" }
print show 2.5
print show "ok"
```

Pipelines, larger:

```curt
us = [{name:"a", score:9, active:true}, {name:"b", score:7, active:false}, {name:"c", score:8, active:true}]
print (us | keep .active | top 2 .score | map .name)
```

String verbs:

```curt
slug s = s.trim.lower.replace " " "-"
print slug "  Hello World  "
```

Accumulator loops:

```curt
steps n = {
  k = 0
  while n != 1 {
    n = if n % 2 == 0 { n / 2 } else { 3*n + 1 }
    k += 1
  }
  k
}
print steps 27
```

A complete expression parser — recursive equations, slices, tuples:

```curt
lex s = ((s.replace "(" " ( ").replace ")" " ) ").words | map w -> if w[0].digit { w.float } else { w }

expr ts = {
  (v, r) = term ts
  while r.len > 0 and r[0] in "+-" {
    (v2, r2) = term r[1:]
    v = if r[0] == "+" { v + v2 } else { v - v2 }
    r = r2
  }
  (v, r)
}

term ts = {
  (v, r) = factor ts
  while r.len > 0 and r[0] in "*/" {
    (v2, r2) = factor r[1:]
    v = if r[0] == "*" { v * v2 } else { v / v2 }
    r = r2
  }
  (v, r)
}

factor ts = match ts[0] {
  float x -> (x, ts[1:]),
  "(" -> { (v, r) = expr ts[1:]; (v, r[1:]) }
}

print (expr (lex args.1)).0
```

The flagship: a JSON-driven log-analytics engine — every feature in one
program (unions, signatures, rescue chains, fs, json, raw strings,
multi-line literals, go):

```curt
type Lat = int | float
type Entry = {svc str, ms Lat}
type Score = {name str, val int}
type Sum = {files int, lines int, bad int}

pub avg :: [Lat] -> float
avg xs = xs.sum / xs.len.float

classify lvl ms why = if lvl == "ERROR" { why } else { match ms.int { err _ -> ms.float, n -> n } }

fetch p = ((fs.read p)?).lines

med xs = {
  s = xs.sort
  s[s.len / 2]
}

bar n = {
  s = ""
  while s.len < n { s += "#" }
  s
}

stat g sc = {
  ms = g.v | map .ms
  print "{g.k}: reqs {g.v.len} avg {avg ms} med {med ms} score {sc}"
}

faults fails = {
  parts = (fails | map .0).counts.pairs | map p -> "{p.k} {p.v}"
  print "errors by svc: {parts | join ', '}"
}

peak mins = {
  best = (mins.counts.pairs | top 1 .v).first
  print "peak minute: {best.k} x{best.v}"
}

lag entries slow = {
  rows = entries | keep e -> e.ms >= slow | map e -> "{e.svc} {e.ms}"
  msg = if rows.len == 0 { "none" } else { rows | join ', ' }
  print "slow >={slow}ms: {msg}"
}

rank groups shown = {
  tops = groups | top shown (g -> g.v.len) | map g -> "{g.k} {g.v.len}"
  print "top {shown} by traffic: {tops | join ', '}"
}

worst scored = {
  w = (scored | top 1 .val).first
  print "worst: {w.name} score {w.val}"
}

hist groups = for g in groups { print "{g.k} {bar g.v.len}" }

footer t = print "scanned {t.files} files, {t.lines} lines, {t.bad} bad"

base = '{"title": "logmill", "sources": ["west.log"]}'
spec = fs.read (args.1 ? "logmill.json") ? base
job = match json spec { err e -> { print "note: bad job spec, using defaults"; json base }, j -> j }
title = job["title"] ? "logmill"
slow = job["slow_ms"] ? 150
shown = job["top"] ? 1
src = job["sources"] ? []

sev = {
  "ERROR": 3
  "WARN": 1
}

entries = []
fails = []
tags = []
mins = []
bad: int = 0
missing = 0
seen = 0

for p in src {
  match fetch p {
    err e -> { missing += 1 }
    ls -> {
      for ln in ls {
        seen += 1
        f = ln.split ","
        if f.len == 5 {
          mins += [f[0][0:5]]
          v = classify f[1] f[3] f[4]
          match v {
            err x -> { bad += 1 }
            str why -> { fails += [(f[2], why)]; tags += [(f[2], f[1])] }
            int n -> { entries += [Entry{svc: f[2], ms: n}]; tags += [(f[2], f[1])] }
            float x -> { entries += [Entry{svc: f[2], ms: x}]; tags += [(f[2], f[1])] }
          }
        } else { bad += 1 }
      }
    }
  }
}

print "== {title} =="
print "files {src.len - missing} missing {missing}"
print "lines {seen} reqs {entries.len} errs {fails.len} bad {bad}"

groups = entries.group .svc
scored = []
for g in groups {
  sc = tags | keep t -> t.0 == g.k | fold 0 acc t -> acc + (sev[t.1] ? 0)
  scored += [Score{name: g.k, val: sc}]
  stat g sc
}

faults fails
peak mins
go lag entries slow
go rank groups shown
go worst scored
go hist groups

total = Sum{
  files: src.len - missing
  lines: seen
  bad: bad
}
footer total
```
