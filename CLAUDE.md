# cmm — project instructions

cmm is a general-purpose, machine-first programming language for AI agents,
optimized for **output-token cost** (the tokenizer is the ISA). Human
readability is a derived view, not a source property. Read DESIGN.md first;
v0.1 in archive/ is the retired action-DSL framing — do not resurrect it.

## Non-negotiable doctrine

- **Measured, never estimated.** Every token-cost claim comes from a real
  tokenizer run (o200k_base now; Anthropic tokenizer once tools/tokens
  lands). If you state a number, a script must reproduce it exactly.
- **Tournament before adoption.** New syntax/stdlib candidates are measured
  against alternatives on the corpus; the loser is recorded, not deleted.
- **Honest negatives are deliverables.** Round-1 of v0.2 lost to Python and
  the doc says so. Keep that standard.
- **Kill criterion is live** (token-bench chunk): model-written cmm within
  ~10pp of Python success rate after one revision cycle, or document the
  negative result and re-scope.
- Identifiers: semantic-but-single-token (`buf`, `idx`, `acc`). Never
  single-letter golf (measured accuracy cost, zero token gain).

## State and tooling

- Plan of record = **native think-and-ship roadmap** (`roadmap_*` MCP tools).
  ROADMAP.md is a generated view — regenerate via `roadmap_export`, never
  hand-edit. Reasoning lives in `think_*` steps (pinned steps are
  load-bearing); execution in `ship_*`.
- Drive work with `/roadmap` (one chunk per invocation); reshape with
  `/roadmap-refresh`. serpapi MCP for research; ministr MCP for code
  exploration when available.
- Token measurement harness currently at `/tmp/cmm-tok/` (venv with
  tiktoken); becomes `tools/tokens` + committed corpus in lang-spec-v01.

## Verification gates

- Docs chunks: claims reproduce via script; files exist; views regenerated.
- Rust chunks (interp-mvp onward): `cargo test && cargo clippy --all-targets
  -- -D warnings`; golden corpus must execute; `cmm tokens` must reproduce
  the spec cost table. Never mask an exit code.

## Repo conventions

- Commit on `main`, imperative subject, body explains the *measured why*.
  End commit messages with: `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`
- `cmm` collides with GHC's C-- (`Cmm`) IR — naming decision is a backlog
  chunk; don't churn the name ad hoc.
