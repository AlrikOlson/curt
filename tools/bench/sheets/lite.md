# curt in 60 seconds

Statements separate by newline. No imports, no main, no semicoled ceremony.

- Bind: `x = 1` · annotate: `x: int = 1` · compound: `x += 1`
- Functions are equations: `f a b = a + b` — call by juxtaposition: `f 1 2`
- Print: `print x` (no parens). Interpolation: `print "n={n}"`
- Lists `[1, 2]`, maps `{"k": 1}` (`m["k"] = v` for dynamic keys), tuples `(a, b)`
- `if c { .. } else { .. }` · `while c { .. }` · `for x in xs { .. }`
- `range n` / `range a b` / `range a b step`
- Pipelines: `xs | map x -> x * 2 | sum` — `|` is loosest; `print` goes OUTSIDE: `print (xs | sum)`
- Verbs (postfix or piped): .len .sum .sort .rev .first .last .min .max .keys .vals
  .upper .lower .trim .split .join .lines .words .chars .counts .int .float .str .json
- Errors: `v = thing?` propagates; `v = thing ? fallback` rescues;
  `match "x".int { err e -> 0, n -> n }`
- Types: `type Pt = {x int, y int}` · `Pt{x: 1, y: 2}` · unions `type V = int | str`
- Files need the fs capability: `fs.read "p"` (run with --fs)
- Concurrency: `go work x`

Reply with only a curt code block.
