<!-- GENERATED VIEW — do not hand-edit. Source of truth is the native think-and-ship
     roadmap (roadmap_* tools / `think-and-ship export`). Regenerated 2026-06-10
     after redesign-v02 completed (v0.2 design shipped, repo initialized). -->

# Roadmap — cmm-d31a18

## Pending

- [ ] **Language spec v0.1 — GP grammar, type system w/ full inference, memory model, measured token-cost table** — Turn DESIGN.md v0.2 into an implementable SPEC.md for the general-purpose language: full PEG/EBNF grammar (ASCII, ceremony-free, no required indentation, LL(1)-friendly for constrained decoding), static type system with FULL inference (annotations optional, 1-token when present), value types/structs/enums/pattern matching, memory model decision (RC vs ownership-inference — token-cost and reliability argued, no lifetime ceremony), terse error model (Result-ish with single-token propagation), identifier policy (semantic-but-single-token; compiler lints identifiers costing >1 token), Postel-parsing acceptance set for Python/Rust drift (arXiv 2503.13620 language-confusion mitigation), structured fix-suggesting errors. Build tools/tokens: per-construct cost table (tiktoken o200k_base + Anthropic count-tokens when key present) + canonical corpus of ~20 REAL program snippets measured against Python AND Go/Rust equivalents; settle every spelling by measured tournament weighted by model-writability evidence. Cost table = CI regression gate thereafter.
  - deps: redesign-v02
  - acceptance: SPEC.md complete enough to implement from; grammar passes a PEG validity check
  - acceptance: tools/tokens runs offline; cost table covers 100% of constructs on both tokenizers (Anthropic when key present)
  - acceptance: Canonical corpus measured: ratios vs Python and vs Go/Rust reported per snippet with medians (target calibration, not cherry-picking)
  - acceptance: Spelling tournaments recorded incl. rejected alternatives
  - acceptance: Type-inference and memory-model decisions argued in token-cost terms with measurements
- [ ] **Reference implementation MVP — compiler front-end + execution backend + CLI (run | fmt | expand | tokens)** — Implement the v0.2 core from SPEC.md in Rust: lexer, PEG parser with Postel acceptance + canonical formatter, type inference engine, and the execution backend chosen in the spec (tree-walk/bytecode VM first; wasm codegen path reserved for wasm-embed). CLI: run, fmt (canonical dense form), expand (the readability-as-view projection: annotated, type-revealed rendering of dense source), tokens (per-tokenizer counts). Structured fix-suggesting diagnostics. cargo workspace at repo root; startup <10ms.
  - deps: lang-spec-v01
  - acceptance: cargo test green with >=40 golden tests incl. inference goldens, Postel-input -> canonical pairs, and expand-view goldens
  - acceptance: cargo clippy --all-targets -- -D warnings clean
  - acceptance: Canonical corpus programs execute correctly
  - acceptance: cmm tokens reproduces the spec cost table exactly
  - acceptance: Startup measured <10ms
- [ ] **The ≤2500-token cheat sheet — measured teachability AND model-legibility** — Compress the GP language into a system-prompt cheat sheet (budget raised to <=2500 tokens for the larger surface; Anthropic tokenizer primary) + few-shot pack. Measure TWO things on >=2 models, honestly reported: (a) teachability — fresh sessions write correct programs for 10 held-out tasks (syntax-validity + semantic-correctness rates); (b) model-legibility — comprehension QA over dense cmm code the model did NOT write (can it answer behavior questions as accurately as over equivalent Python? — this guards the machine-first surface against the naming/structure comprehension findings). Iterate sheet wording (not the language) up to 3 rounds.
  - deps: interp-mvp
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
- [ ] **Constrained-decoding artifacts — GBNF + llguidance/XGrammar grammars + zero-error demo** *(re-prioritization proposed: 60 → 25, user decision pending)* — Generate GBNF (llama.cpp) and llguidance/XGrammar grammar artifacts from the spec grammar (single source of truth — derive, don't hand-maintain). Demo with an OSS runtime (vLLM or llama.cpp) showing grammar-masked generation produces 0% parse errors on a generation suite. Investigate + document honestly what is possible on closed APIs (Anthropic/OpenAI) via prefill/stop-sequence conventions vs true CFG support — a negative finding here is a finding.
  - deps: lang-spec-v01, interp-mvp
  - acceptance: Grammar artifacts generated from the spec source, with a divergence test proving they accept exactly the golden corpus
  - acceptance: OSS-runtime demo: 0 parse errors across >=200 constrained generations
  - acceptance: Closed-API capability matrix documented (what each vendor can/cannot enforce in 2026)

## Done

- [x] **DESIGN.md v0.2 — general-purpose machine-first language (user-directed pivot)** — Shipped 2026-06-10 (commit 5dfe9a8; proof: task:verify-v02). v0.2 measured both design rounds: Python parity (1.02×) at 2.08×/2.25× vs Go/Rust; round-1 loss autopsy produced the dense-stdlib rule and untagged unions (the ADT tax); Vera/NanoLang/MoonBit differentiated from primary sources (they spend tokens on machine trust; cmm saves them). **User sign-off on v0.2 direction still pending — it gates lang-spec-v01.**

## Backlog

- [ ] **Host interface — C FFI, wasm imports, tool/LLM access as stdlib (not grammar)** — Replaces the obsoleted agent-prims chunk under the v0.2 framing: agent capabilities (tool calls, LLM calls, retry/parallel helpers) become a host-interface LIBRARY over C FFI / wasm imports, not language grammar. The language stays a clean general-purpose core; agent affordances are its standard host bindings (MCP-compatible registry injection preserved as a library concern). Budget caps and structured rescuable errors carry over as library/runtime features.
  - deps: interp-mvp
  - acceptance: C FFI + wasm import surface specified and implemented for the reference runtime
  - acceptance: Tool-registry library demonstrated end-to-end against a mock host
  - acceptance: ask/llm-call helper library with shape validation works against a mock backend
- [ ] **MCP server distribution wedge — run_cmm tool whose description is the cheat sheet** — Ship cmm as an MCP server exposing run_cmm(program, args?) with the cheat sheet as the tool description — the zero-install adoption wedge stays valid under v0.2 (agents get a sandboxed wasm-backed GP language as a tool call). Tool/LLM access flows through the host-ffi library layer rather than language grammar.
  - deps: host-ffi, cheatsheet
  - acceptance: MCP server runs under a real client (e.g. Claude Code) and executes cmm programs end-to-end
  - acceptance: Tool description fits the cheat-sheet budget and a fresh agent session uses cmm unprompted for a glue task
  - acceptance: Registry pass-through demonstrated: cmm program calls a tool provided by the host client
- [ ] **wasm32-wasi build + JS/Python embeddings** — Compile the interpreter to wasm32-wasi; publish a JS package (browser/edge/Workers sandboxes) and a pyo3 Python module so CodeAct-style frameworks (smolagents, LangGraph) can swap cmm in as the action runtime without a native dependency.
  - deps: interp-mvp
  - acceptance: wasm build passes the golden test suite
  - acceptance: npm + PyPI packages execute the canonical corpus
  - acceptance: One smolagents-or-equivalent integration example runs
- [ ] **Naming + identity decision (cmm vs C-- collision)** — USER DECISION REQUIRED: `cmm` collides with GHC's C-- intermediate representation (Cmm, .cmm files). Decide keep-with-pun (comm/communication, "less than C" — the pun got BETTER under the low-level v0.2 framing) vs rebrand (candidates: pith, terse, lac). Check domain + crate/npm/PyPI name availability, pick file extension, write the one-paragraph identity statement.
  - acceptance: User has made the name call
  - acceptance: Domain/registry availability checked and recorded
  - acceptance: README/DESIGN/SPEC renamed consistently if changed
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

- [-] **Agent primitives — host tool registry, ask {shape}, ? retry/skip, par N, budgets** — *Obsoleted 2026-06-10: user redirect — cmm is a general-purpose language, not an exec action language; grammar-level agent verbs rejected. Successor: host-ffi (backlog).*
