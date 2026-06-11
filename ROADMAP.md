<!-- GENERATED VIEW — do not hand-edit. Source of truth is the native think-and-ship
     roadmap (roadmap_* tools / `think-and-ship export`). Regenerated 2026-06-10
     after accepted reprios: naming P33 + playground P70, both pending. -->

# Roadmap — cmm-d31a18

## Pending

- [ ] **grammar-decode B/OpenAI — custom-tools CFG conformance run** *(BLOCKED: needs OPENAI_API_KEY, probed absent 2026-06-10)* — Submit the real cmm Lark grammar to OpenAI custom tools; measure conformance over >=100 generations; document size/complexity limits.
  - deps: gd-a
- [ ] **Naming + identity decision (cmm vs C-- collision)** *(accepted reprio 120→33, now pending — leads the launch runway)* — USER DECISION: keep `cmm` (collides with GHC's C-- IR) vs rebrand (pith/terse/lac); check domain + registry availability; pick file extension; one-paragraph identity statement.
- [ ] **OSS foundation — dual license, CI running every gate, contributor hygiene (AAA repo baseline)** *(added 2026-06-11, think:29)* — Repo has NO LICENSE/CI/CONTRIBUTING. MIT OR Apache-2.0 dual; GitHub Actions running ALL gates (cargo, PEG, lark+negatives, count regression, GBNF determinism); CONTRIBUTING.md encoding the doctrine; release binaries.
  - deps: interp-d
  - acceptance: CI green on fresh checkout, every gate, none weakened
- [ ] **Hugging Face publish — org, dataset repo, GitHub mirror, launch** *(added 2026-06-11, think:29)* — AFTER token-bench (launch carries measured numbers) and naming (gates public identifiers). Dataset repo: corpus + grammars + task suites + results; measured-claims-only launch.
  - deps: token-bench, oss-foundation, naming
- [ ] **The ≤2500-token cheat sheet — measured teachability AND model-legibility** — Compress the GP language into a system-prompt cheat sheet (budget raised to <=2500 tokens for the larger surface; Anthropic tokenizer primary) + few-shot pack. Measure TWO things on >=2 models, honestly reported: (a) teachability — fresh sessions write correct programs for 10 held-out tasks (syntax-validity + semantic-correctness rates); (b) model-legibility — comprehension QA over dense cmm code the model did NOT write (can it answer behavior questions as accurately as over equivalent Python? — this guards the machine-first surface against the naming/structure comprehension findings). Iterate sheet wording (not the language) up to 3 rounds.
  - deps: interp-d
  - acceptance: Cheat sheet measured <=2500 tokens on both tokenizers
  - acceptance: Teachability measured across >=2 models, reported whatever the numbers are
  - acceptance: Model-legibility QA: comprehension accuracy within 5pp of same-program-in-Python or documented as a design problem feeding back to spec
  - acceptance: Sheet is the canonical source for the future MCP tool description
- [ ] **The benchmark — output tokens AND success rate vs Python/Go/Rust (moment of truth)** — 15-20 REAL programming tasks (algorithms, data structures, a parser, text/data processing, a small multi-file module — HumanEval-class plus systems-flavored; EffiBench-X as harness prior art) with executable verifiers. For each task and language (cmm w/ cheat sheet, Python, Go, Rust; JS optional): same prompt, model generates, harness executes; record output tokens (o200k_base + Anthropic) and pass/fail; >=2 models, >=3 samples per cell. Report full distributions; split structure-heavy vs payload-heavy. THREE metrics: (1) output tokens per solved task; (2) success rate; (3) INPUT-side re-read cost — tokens to hold the equivalent codebase in context (the compounding economics for maintained software). CARRIES THE KILL CRITERION: if cmm success is not within ~10pp of Python after one language-revision cycle, document the negative result and re-scope. One revision cycle in-scope.
  - deps: cheatsheet
  - acceptance: BENCHMARK.md with all cells reported (no cherry-picking), per-model and per-task distributions
  - acceptance: Output-token ratios vs Python AND Go/Rust with medians + spread; RECALIBRATED targets (redesign-v02 measurements): parity-to-1.2x vs Python by task mix, >=1.8x vs Go (2.08x measured on the design corpus) — evaluated honestly
  - acceptance: Input-side re-read cost measured on the multi-file task
  - acceptance: Kill criterion explicitly evaluated PASS/FAIL per model
  - acceptance: Harness re-runnable with one command

## Done

- [x] **grammar-decode B/OSS — llama.cpp constrained-decoding demo (local 8B)** — Shipped 2026-06-11 (commit ea6db8e; proof: task:verify-gdb). MASK SOUNDNESS: 0 mid-stream violations / 200 constrained generations (all 31 failures valid prefixes — non-termination only) vs 55 real mid-stream errors in 100 unconstrained. Haiku: naive 7/10; self-taught from repo artifacts 10/10. Four findings: 30% keyword-widening leak → exact prefix-trie NAME in GBNF; llama.cpp grammar-state perf cliff; the termination problem + measured EOS-escape; a real arg-position grammar divergence (if/match as args) fixed across PEG+Lark+GBNF (negatives 12/12).
- [x] **interp D — evaluator + corpus stdlib + capability IO; cmm run executes the corpus** — Shipped 2026-06-10 (commit 7729652; proof: task:verify-d). 126/126 tests; all 20 corpus programs execute with golden stdout incl. a live TCP smoke test; run-startup 2.06ms; clippy clean. New shared elaboration rule: rescue captures like pipes (`print m["k"] ? 8080` ≡ `print (m["k"] ? 8080)`). Golden-caught bugs: quote-polluted string lexemes (FNV hashed quote bytes), list numeric fields, UInt operand order. Honest limit: `go` is sequential in v0.1 (Rc⇏Send; threads deferred §13). THE INTERPRETER IS COMPLETE: parse | check | fmt | expand | tokens | run.
- [x] **grammar-decode A — Lark CFG twin + GBNF artifact + machine divergence gate** — Shipped 2026-06-10 (commit 46dc539; proof: task:verify-gda). cmm.lark (explicit whitespace — SPEC §1 adjacency survives as grammar structure; three PEG lookaheads dissolve in CFG form) validated 20/20 corpus + 10/10 negative agreement with the Rust oracle, both first run. cmm.gbnf generated deterministically (terminals pinned; documented keyword-widening — GBNF lacks lookahead). Chain of trust: Rust parser ⇄ grammar.peg ⇄ cmm.lark → cmm.gbnf. Parent grammar-decode obsoleted by split (gd-b carries the model/API demo legs).
- [x] **interp C — type inference (arity resolution, unions, narrowing) + expand type-reveal + diagnostics** — Shipped 2026-06-10 (commit 2f5e739; proof: task:verify-c). 106/106 tests (27 new goldens), clippy clean, `cmm check` passes 20/20 corpus, startup 3.33ms. Both §2.3 elaboration rules fixed (arity re-nesting, pipe capture); UFCS receiver-last; union narrowing + exhaustiveness with fix-suggesting JSON diagnostics; int→float widening at argument positions only. THE CHECKER CAUGHT A REAL PRECEDENCE BUG in 20_server (31→32 tokens; medians re-measured: Python 1.19× unchanged, Go 2.38→2.34×) — the trust dividend, demonstrated. Parenthesized-lambda parser gap fixed; grammar.peg synced.
- [x] **interp B — canonical formatter (fmt) + sugar-expand skeleton** — Shipped 2026-06-10 (commit 4ff4241; proof: task:verify-b). 79/79 tests all green first run; fmt is byte-identical on the canonical corpus (token delta 0), idempotent, parse-preserving; Postel goldens ×9; adjacency round-trips proven; comments/blank-lines preserved via trivia-aware lex_raw; SPEC §1 amended: adjacency is a first-class lexical rule. Known limitation recorded: expression-position `=` not rewritten to `==` until interp-c.
- [x] **interp A — cargo skeleton, lexer, parser (Postel), AST; cmm parse|tokens; corpus 20/20 in Rust** — Shipped 2026-06-10 (commit b2ca8a6; proof: task:verify-a). 60/60 tests, clippy clean, exact `cmm tokens` parity with count.py on 20/20 corpus files, startup 2.66ms. Discovery: adjacency (gluedness) is a first-class semantic channel — field-vs-projection, propagate-vs-rescue, literal/call-sugar-vs-juxtaposition — SPEC §1 amendment queued for interp-b.
- [x] **Language spec v0.1 — GP grammar, type system w/ full inference, memory model, measured token-cost table** — Shipped 2026-06-10 (commit 8e74b9e; proof: task:verify-spec). SPEC.md implementable; PEG grammar machine-validated 20/20 against the 52-file canonical corpus; medians 1.19× vs Python (n=20, wins 13/ties 2/loses 5 — reported), 2.38×/2.69× vs Go/Rust (n=6 flagged); tournaments recorded with losers (float>flt BY COST, range>.., pub>::-export); RC memory decided on measurement (ownership ceremony ≥5% of Rust corpus tokens). tools/tokens/{count,validate}.py are permanent CI gates.
- [x] **DESIGN.md v0.2 — general-purpose machine-first language (user-directed pivot)** — Shipped 2026-06-10 (commit 5dfe9a8; proof: task:verify-v02). v0.2 measured both design rounds: Python parity (1.02×) at 2.08×/2.25× vs Go/Rust; round-1 loss autopsy produced the dense-stdlib rule and untagged unions (the ADT tax); Vera/NanoLang/MoonBit differentiated from primary sources (they spend tokens on machine trust; cmm saves them). v0.2 direction approved by user (recorded at lang-spec-v01 start).

## Backlog

- [ ] (backlog) **py2cmm — Python-subset transpiler + differential-execution data pipeline (MultiPL-T pattern)** — DECIDED route for training data (2026-06-11, think:32): transpile MBPP/HumanEval-class test-paired Python; filter by python3 == `cmm run` + `cmm check` + fmt-canonical; emit JSONL with provenance. Proc-gen demoted to fuzzing side-stream (gd-b-oss measured why: syntax-valid nonsense). Feeds finetune-probe and the hf-publish dataset (optional enrichment). P85, deps interp-d.
- [ ] (backlog) **Hosted fine-tune probe — put cmm INTO weights (no local training)** — Bedrock Claude 3 Haiku SFT / OpenAI hosted / Together-Fireworks OSS; fine-tune + grammar-mask combination hypothesis; deps cheatsheet. User-signaled 2026-06-11.
- [ ] **Host interface — C FFI, wasm imports, tool/LLM access as stdlib (not grammar)** — Replaces the obsoleted agent-prims chunk under the v0.2 framing: agent capabilities (tool calls, LLM calls, retry/parallel helpers) become a host-interface LIBRARY over C FFI / wasm imports, not language grammar. The language stays a clean general-purpose core; agent affordances are its standard host bindings (MCP-compatible registry injection preserved as a library concern). Budget caps and structured rescuable errors carry over as library/runtime features.
  - deps: interp-d
  - acceptance: C FFI + wasm import surface specified and implemented for the reference runtime
  - acceptance: Tool-registry library demonstrated end-to-end against a mock host
  - acceptance: ask/llm-call helper library with shape validation works against a mock backend
- [ ] **MCP server distribution wedge — run_cmm tool whose description is the cheat sheet** — Ship cmm as an MCP server exposing run_cmm(program, args?) with the cheat sheet as the tool description — the zero-install adoption wedge stays valid under v0.2 (agents get a sandboxed wasm-backed GP language as a tool call). Tool/LLM access flows through the host-ffi library layer rather than language grammar.
  - deps: host-ffi, cheatsheet
  - acceptance: MCP server runs under a real client (e.g. Claude Code) and executes cmm programs end-to-end
  - acceptance: Tool description fits the cheat-sheet budget and a fresh agent session uses cmm unprompted for a glue task
  - acceptance: Registry pass-through demonstrated: cmm program calls a tool provided by the host client
- [ ] **wasm32-wasi build + JS/Python embeddings** — Compile the interpreter to wasm32-wasi; publish a JS package (browser/edge/Workers sandboxes) and a pyo3 Python module so CodeAct-style frameworks (smolagents, LangGraph) can swap cmm in as the action runtime without a native dependency.
  - deps: interp-d
  - acceptance: wasm build passes the golden test suite
  - acceptance: npm + PyPI packages execute the canonical corpus
  - acceptance: One smolagents-or-equivalent integration example runs
- [ ] **tree-sitter grammar + syntax highlighting** — tree-sitter-cmm derived from the spec grammar, highlight queries, and a minimal VS Code extension — humans debug cmm even if they don't write it; readable traces matter for trust.
  - deps: lang-spec-v01
  - acceptance: tree-sitter parses the full golden corpus with zero errors
  - acceptance: Highlighting visually verified on the canonical examples
- [ ] **Web playground with live token meter (cmm vs Python side-by-side)** — Browser REPL (wasm runtime) showing a cmm program and its Python equivalent side-by-side with live per-tokenizer counts and cost deltas — the demo that makes the value proposition visceral. GUI work: follows /craft §B (tokens, Storybook, /gui-scrutiny verify).
  - deps: wasm-embed
  - acceptance: Playground runs the canonical corpus in-browser via wasm
  - acceptance: Live token meter matches tools/tokens output exactly
  - acceptance: GUI passes /gui-scrutiny (light+dark, mechanical assertions)
- [ ] **Stdlib v2 — dates, regex, csv, math (measured admission only)** — Second wave of stdlib functions admitted strictly by the cost-table process: each candidate must demonstrate a recurring task pattern where its absence costs more tokens than its cheat-sheet line costs to teach. Keeps the language small under growth pressure.
  - deps: token-bench
  - acceptance: Every admitted function has a measured before/after on a real task pattern
  - acceptance: Cheat sheet stays within budget after additions

## Obsoleted

- [-] **Constrained-decoding artifacts (grammar-decode)** — *Obsoleted 2026-06-10: split into gd-a (local artifacts + gate, done) and gd-b (environment-gated demos, pending P27).*
- [-] **Reference implementation MVP (interp-mvp)** — *Obsoleted 2026-06-10: split into interp-a/b/c/d (multi-session scope, per skill discipline); acceptance criteria distributed across the four sub-chunks.*
- [-] **Agent primitives — host tool registry, ask {shape}, ? retry/skip, par N, budgets** — *Obsoleted 2026-06-10: user redirect — cmm is a general-purpose language, not an exec action language; grammar-level agent verbs rejected. Successor: host-ffi (backlog).*
