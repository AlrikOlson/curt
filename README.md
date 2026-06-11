# curt

A **general-purpose, machine-first programming language** for AI agents:
statically typed, compiled (wasm-first), with **output-token cost as the prime
design directive** — human readability is a derived view (`curt expand`), not a
property of source.

> **Why "curt"?** Brief to the point of bluntness — a language too
> token-thrifty for pleasantries. The name keeps a quiet C lineage (*C,
> abbreviated*): the project began as `cmm`, an intentional C-- homage, and
> was renamed to leave GHC's `Cmm` IR its namespace. Every claim about curt
> is measured, never estimated; the name was chosen the same way (1 token
> mid-prompt under o200k; bare `crates.io/curt` free — availability verified
> 2026-06-11).

```curt
handle c = for ln in c.lines { c.write (ln.upper + "\n") }
for c in net.listen 8080 { go handle c }
```

A concurrent TCP uppercase-echo server in **32 tokens** (o200k_base).
Python: 55 · Go: 94 · Rust: 123.

Measured across three real programs (word-frequency, expression parser,
concurrent server): **token parity with Python (1.02×) at 2.08× cheaper than
Go and 2.25× cheaper than Rust** — compiled-language semantics at
dynamic-language emission cost. Numbers, methodology, and the round our first
draft *lost* are all in **[DESIGN.md](DESIGN.md)**.

Core ideas: Haskell-style equations + juxtaposition (zero definition/call
ceremony), full type inference with **untagged unions** (the measured answer
to the ADT tax), a deliberately dense single-token stdlib, flat KV-cache-
friendly structure, semantic-but-single-token identifiers (naming research
says obfuscation costs accuracy and buys nothing), grammar shipped as
constrained-decoding artifacts (syntax errors become impossible), and
RC-managed memory with zero token ceremony.

**Status:** specified — **[SPEC.md](SPEC.md)** is implementable (grammar
machine-validated 20/20 against [corpus/](corpus/); cost table reproducible via
`tools/tokens/count.py`; corpus medians: **1.19× vs Python** (n=20), 2.34×/2.69×
vs Go/Rust). Next: the reference implementation. The build plan lives in the
native think-and-ship roadmap; [ROADMAP.md](ROADMAP.md) is its generated view.
The retired v0.1 action-DSL framing is archived in
[archive/](archive/DESIGN-v0.1.md).
