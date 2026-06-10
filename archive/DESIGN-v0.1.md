# cmm — a language for agents, measured in tokens

> **⚠ SUPERSEDED (2026-06-10):** this v0.1 framed cmm as an *action/orchestration DSL*.
> Per user direction, cmm is being redesigned as a **general-purpose, machine-first
> programming language** (real programs, not glue; human readability deprioritized).
> The measured methodology, tokenizer findings, and kill-criterion discipline below
> carry forward; the wedge (§2) and agent-verbs-in-grammar (§3 P5) do not.
> See roadmap chunk `redesign-v02` for the v0.2 rewrite.

**Status:** design v0.1 (2026-06-10) · **Working title:** `cmm` (pronounced *"comm"*; see [Open questions](#open-questions) for the naming collision)

> `cmm` is an executable action language for AI agents whose **express design goal is
> minimizing output tokens**. The BPE tokenizer is treated as the instruction-set
> architecture: every keyword, operator, and grammar rule is chosen against its
> measured token cost, and the spec ships with a token-cost table enforced in CI.

```cmm
get "https://api.shop.io/users" | json
| keep .active & .role = "admin"
| top 5 by .age | map .email
```

34 tokens (o200k_base). The idiomatic Python equivalent is 72. Same behavior, 2.12× cheaper to emit.

---

## 1. Why this language should exist

Four facts about the 2026 agent economy, all verified against current sources:

1. **Output tokens are the expensive direction.** Across major providers, output
   tokens are priced at **3–5× input tokens** (up to 8× on reasoning models — e.g.
   GPT‑5.2 at $1.75/M in vs $14.00/M out). Industry analyses now call output-heavy
   agent patterns "the primary cost driver in production deployments."
2. **Output tokens are also the latency.** Decoding is serial; an agent step that
   emits 120 tokens of code takes roughly twice as long as one that emits 60. In a
   30-step loop, emission time dominates wall-clock.
3. **Emitted code compounds.** Every program an agent writes re-enters the context
   window as history on every subsequent turn ("naive agent loops rebill prior
   context on every call"). Halving emission size shrinks both the output bill *and*
   the growing input bill of every later step.
4. **Code actions beat JSON tool calls — and the code is Python by default.**
   CodeAct (ICML 2024) showed agents emitting executable code outperform JSON tool
   calling by up to 20 points of success rate with ~30% fewer steps. The entire
   ecosystem (OpenHands, smolagents, LangGraph-codeact) standardized on **Python** —
   a language designed for human ergonomics in 1991, not for token-priced emission
   in 2026.

The gap: **TOON** fixed the *data* side (30–60% fewer tokens than JSON for structured
input). **Nobody has built the *code* side** — an executable language designed against
a measured token budget. Existing general-purpose languages span a 2.6× token-efficiency
spread on identical tasks (RosettaCode study, Jan 2026), and none of them were
*designed* for this axis. That spread is the proof that the axis is real; `cmm` is the
attempt to win it on purpose.

## 2. The wedge: code-as-action, not code-as-artifact

Armin Ronacher's *A Language For Agents* (Feb 2026) argues agent-era languages should
be **more explicit** — spelled-out types, no inference — because durable code is read,
reviewed, and maintained. He is right *for software*. But most code an agent emits is
not software: it is a **disposable action** — fetch, filter, call a tool, extract,
emit — executed once in a sandbox and discarded like a tool-call payload. Nobody
reviews it. Its only readers are the interpreter and (as history) the model itself.

For that layer the optimization flips: every token is pure cost. `cmm` claims exactly
that layer and concedes the durable-software layer to Python/TypeScript:

| | code-as-artifact | code-as-action |
|---|---|---|
| lifetime | years | one execution |
| reader | humans + agents | interpreter only |
| optimization | clarity, reviewability | tokens, reliability |
| right language | Python, TS, Rust… | **cmm** |

When a task outgrows glue (real algorithms, data structures), the agent should write
Python. `cmm` is the default for the ~80% of agent steps that are orchestration.

## 3. Design pillars

**P1 — The tokenizer is the ISA.** Constructs are selected by *measured* cost under
o200k_base and the Anthropic tokenizer, not by character count. Character density is
not token density: APL's `⌽⍳5` is 3 characters but **6 tokens**; cmm's `top 5 by .age`
is 13 characters but also 6 tokens — and the model has seen English words a trillion
times. All 25 core keywords (`map keep sort top by ask par fn if else join get post
json out retry not and or in first last flat group pick`) and every operator
(`| . , : ; ( ) [ ] { } = < > + - * / & ? ! => -> >= <= != ..`) are verified
single-token. The spec ships a cost table; CI fails any grammar change that regresses
the canonical corpus.

**P2 — Pipeline-first, point-free.** Data flows left→right through `|` (the
jq/shell mental model — abundantly present in model weights). The current value is the
implicit subject: `.field` reads from it, bare `.` is the value itself. This kills the
two biggest token sinks in emitted Python: intermediate variable naming
(`result = …` … `result`) and lambda ceremony (`lambda u: u["email"]` → `.email`,
10 tokens → 3). Pipelines also flatten nesting, so a model never has to balance
brackets more than ~2 deep — a known LLM failure mode.

**P3 — No required indentation, no closing ceremony.** Newline and `;` are statement
separators (1 token each); blocks are pipeline stages or parenthesized expressions.
Measured: `"\n"` = 1 token, `"\n    "` = 2 — every indented line in Python costs an
extra token, and (Ronacher) significant whitespace is the thing models corrupt during
surgical edits. There is no `end`, no mandatory braces, no `return` (last expression
is the result), no `print` for the final value (programs evaluate to their output).

**P4 — JSON-superset values, shape literals.** Values are null/bool/num/str/list/obj
(+ lazy streams). Path expressions `.users[0].email`, slices `[:5]`, string
interpolation `"CEO of {company}"`. A **shape literal** `{name, year:int}` is an
inline schema: used with `ask` it validates + coerces, used with `expect` it asserts.
Structured data is first-class because agent glue is 90% reshaping JSON.

**P5 — Agent verbs are primitives, not libraries.**
- **Tool calls** use command syntax wired to the host's tool registry (MCP-compatible):
  `web.search "rust 2026" n:5`, `mail.send to:"x@y.z" subject:s body:.` —
  positional args then `k:v` named args, no parens/commas in pipeline position.
- **`ask`** is the LLM call: `ask "summarize: {.}"` → text;
  `ask {name, year:int} "who founded {co}?"` → schema-constrained extraction,
  validated and coerced. 10 tokens for what costs Python ~27
  (llm + json.loads + assert) — and the runtime does constrained decoding on the
  sub-call, so it cannot return malformed JSON.
- **`?` rescue** is postfix error handling: `parse json ? {}` (fallback value),
  `http get u ? retry 2` (bounded retry), `? skip` (drop element in map context).
  Measured: ` ? retry 2` = **4 tokens** vs Python's retry loop = **24**.
- **`par N`** is bounded parallel map: `urls | par 8 map fetch`. Python's
  ThreadPoolExecutor dance is ~25 tokens of ceremony.
- **`fn`** defines the rare reusable helper: `fn slug(s): s | lower | replace " " "-"`.

**P6 — Designed for constrained decoding.** The grammar is PEG, LL(1)-friendly, ASCII
only, and ships as GBNF / llguidance / XGrammar artifacts. Runtimes that support
grammar-constrained sampling (vLLM, llama.cpp, SGLang — commodity infrastructure in
2026) can make cmm syntax errors **physically impossible to emit**. JSON Schema
constrained decoding made malformed JSON extinct; a small regular language can get the
same guarantee for whole programs. Python can never have this — its grammar (and the
arbitrary stdlib surface behind it) is too large to mask meaningfully.

**P7 — Postel parsing: accept the model's accent.** Models carry Python muscle
memory. Where unambiguous, the parser accepts the slip and the formatter
canonicalizes: `==` for `=`, `and/or/not` for `&`/`or`/`not`, `True/False/None`,
trailing commas, smart quotes. Recoverable habits are never errors. This directly
buys generation reliability without costing the canonical form any tokens.

**P8 — Errors are prompts.** Runtime/parse errors return structured, fix-suggesting
values: `{err:"unknown_field", at:"1:12", got:".emial", near:[".email"], fix:"use .email"}`.
The error message is designed to be fed straight back to the model for one-shot
self-repair — the error surface is part of the language's UX, because the "user" is
an agent in a loop.

## 4. Syntax tour (the whole language fits here)

```cmm
# comments with # (agents are told not to emit them)
x: get "https://a.io/data" | json     # binding: name + colon (statement position)
x | keep .score >= 7 & .lang = "en"   # predicates: = != < <= > >= & or not, in
| sort by .ts | first 10              # verbs: sort/top/first/last/flat/group/pick/join
| map {id:.id, t:.title | upper}      # object literal; nested pipeline in value
| out                                  # explicit mid-pipe emit (final value auto-emits)

if .n > 0: "yes" else: "no"           # if/else are expressions
items | each: mail.send to:.addr      # each = side-effecting map
fn dom(u): u | split "/" | .[2]       # function definition
data | expect {id, items:[{sku}]}     # shape assert (fails structured)
ask [str] "list the risks in {.}"     # ask with array-of-string shape
```

Value model: JSON + streams + functions. Numbers are i64/f64 (JSON-compatible).
Equality is `=` (Postel: `==` accepted). Binding `name:` only at statement start;
`k:v` elsewhere is a named argument or object field — position disambiguates (LL(1)).

## 5. Worked examples — measured, not estimated

All counts: o200k_base via tiktoken, measured 2026-06-10 (harness: `tools/tokens`,
to be committed with the spec chunk). Python baselines are *idiomatic concise* Python
(comprehensions, f-strings), not strawmen.

### Ex1 — fetch → filter → top-N → project *(2.12×)*

```python
import requests
users = requests.get("https://api.shop.io/users").json()
admins = [u for u in users if u["active"] and u["role"] == "admin"]
top5 = sorted(admins, key=lambda u: u["age"], reverse=True)[:5]
print([u["email"] for u in top5])
```
**Python: 72 tokens**

```cmm
get "https://api.shop.io/users" | json
| keep .active & .role = "admin"
| top 5 by .age | map .email
```
**cmm: 34 tokens**

### Ex2 — multi-tool: search → extract (LLM) → mail, with retry *(1.84×)*

```python
import json
data = None
for attempt in range(3):
    try:
        r = web_search(f"{company} CEO 2026")
        text = "\n".join(h["snippet"] for h in r["results"][:5])
        raw = llm(f"Extract CEO name and start year as JSON: {text}")
        data = json.loads(raw)
        if "name" in data:
            break
    except Exception:
        continue
send_email(to="me@x.com", subject=f"CEO of {company}", body=json.dumps(data))
```
**Python: 118 tokens**

```cmm
web.search "{company} CEO 2026" | .results[:5] | map .snippet | join "\n"
| ask {name, year:int} "CEO of {company}?" ? retry 2
| mail.send to:"me@x.com" subject:"CEO of {company}" body:.
```
**cmm: 64 tokens**

### Ex3 — parallel batch: summarize every doc, write an index *(1.73×)*

```python
import glob
from concurrent.futures import ThreadPoolExecutor
def summarize(path):
    text = open(path).read()
    return f"- {path}: {llm(f'One-line summary: {text}')}"
with ThreadPoolExecutor(4) as ex:
    lines = list(ex.map(summarize, glob.glob("docs/*.md")))
open("docs/INDEX.md", "w").write("\n".join(lines))
```
**Python: 88 tokens**

```cmm
fs.list "docs/*.md"
| par 4 map {f:., s: fs.read . | ask "one-line summary"}
| map "- {.f}: {.s}" | join "\n" | fs.write "docs/INDEX.md"
```
**cmm: 51 tokens**

### Construct-level costs (where the leverage lives)

| construct | cmm | Python equivalent | factor |
|---|---|---|---|
| bounded retry w/ fallback | ` ? retry 2` — **4** | try/except/range loop — **24** | 6.0× |
| LLM extract + validate | `ask {name, year:int} "…"` — **10** | llm + json.loads + asserts — **27** | 2.7× |
| field-projection lambda | `map .email` — **3** | `map(lambda u: u["email"], …)` — **10** | 3.3× |

**Honest reading of the numbers.** Full-program ratios land at **1.7–2.1×**, not the
construct-level 3–6×, because irreducible payload (URLs, prompts, string literals)
dilutes structural savings — the more payload-heavy the task, the lower the ratio.
The claim this project makes and must defend: **≥1.7× median output-token reduction
on a real agent-task suite, at success-rate parity.** Not 10×. The compounding
effects (§1.2–1.3) multiply whatever per-program factor survives measurement.

## 6. Execution model

- **Runtime:** a small Rust interpreter (`cmm run`, `cmm fmt`, `cmm tokens`), startup
  <10 ms, embeddable; `wasm32-wasi` build so any sandbox (browser, edge, Workers,
  microVM) can execute it. No packages, no imports — capability comes from the
  **host tool registry** (JSON-Schema/MCP tool definitions injected at startup), which
  is also the security boundary: a cmm program can only do what its registry exposes.
- **Determinism:** evaluation is deterministic given tool results; `par` preserves
  output order (results ordered by input index, not completion).
- **Budgets:** the host can cap wall-time, tool calls, and `ask` spend per run;
  exceeding a cap raises a structured, rescuable error.

## 7. Distribution: the language is a tool call away

Languages die on adoption. `cmm` ships as an **MCP server** exposing one tool —
`run_cmm(program, args?)` — whose tool description *is* the cheat sheet: a ≤1,500-token
compressed spec with few-shot examples. Any MCP-capable agent can start emitting cmm
the moment the server is registered; with prompt caching, the cheat sheet is paid
once and amortizes across every step of every session (the TOON paper's
"instructional overhead" critique, answered by caching + the wedge being *output*
tokens). No installs, no training, no buy-in beyond a config line.

## 8. Prior art and what's actually novel

| project | what it is | relation |
|---|---|---|
| CodeAct / OpenHands / smolagents | agents emit Python as actions | the incumbent; cmm targets its emission cost |
| TOON / TRON | token-efficient *data* formats (input side) | complementary; cmm is the code side |
| jq | dense pipeline data language | closest syntactic cousin; data-only, no tools/LLM/effects, cryptic corners |
| K/Q/APL | maximal *character* density | tokenizer-hostile (measured), near-zero weights presence |
| LMQL / Guidance / DSPy | host-side DSLs constraining LLM output | different layer; their constrained-decoding infra is cmm's substrate |
| SudoLang & pseudocode prompting | informal "languages" for prompts | not executable, no measured guarantees |
| Ronacher's *A Language For Agents* | manifesto for durable agent-era languages | the artifact layer; cmm takes the action layer |

Novel synthesis (no existing project does any of these, let alone all):
**(a)** tokenizer-cost-audited grammar with regression CI, **(b)** executable
agent primitives (`ask`/tools/`?`/`par`) as language constructs, **(c)** grammar
shipped as a constrained-decoding artifact making emission errors impossible,
**(d)** distribution as an MCP tool whose description teaches the language.

## 9. Risks, honestly

1. **The weights gap (the big one).** Python enjoys trillions of training tokens;
   cmm has zero. Mitigations: jq/shell-adjacent syntax (high weights overlap),
   Postel parsing, ≤1.5k-token cached cheat sheet, constrained decoding as a
   floor. **Kill criterion:** if, after one language-revision cycle, model-generated
   cmm cannot reach success-rate within ~10 points of model-generated Python on the
   benchmark suite, this project documents the negative result and re-scopes (e.g.
   to constrained-decoding-only deployments, or to a Python-subset transpiler).
2. **Payload dominance.** Tasks dominated by string payload see ratios → 1×. The
   benchmark must report the distribution, not the best case.
3. **Tokenizer drift.** Costs differ across vendor tokenizers and change over time.
   The cost table is computed per-tokenizer (o200k_base + Anthropic API counts);
   design decisions require winning on both, betting on the stable property
   (common English words + ASCII ops are cheap everywhere).
4. **Cheat-sheet overhead on short sessions.** One-shot tasks may not amortize 1.5k
   input tokens. cmm's economics need loops ≥ a few steps — exactly where agents
   already are. (And cached input is an order of magnitude cheaper than output.)
5. **Expressiveness cliff.** Real algorithms in cmm would be miserable. Non-goal by
   design (§2); the cheat sheet tells the model when to fall back to Python.

## 10. Non-goals

- Not a general-purpose language; no classes, no modules, no package manager.
- Not for humans to write (pleasant to *read* is enough — debugging happens).
- Not a prompting DSL; cmm executes, it does not template prompts.
- Not chasing maximum theoretical density (that's APL's grave); chasing *measured
  cost under real tokenizers at acceptable reliability*.

## 11. Open questions

1. **Name.** `cmm` collides with C-- ("Cmm"), GHC's intermediate language (`.cmm`
   files). Candidates if rebranding: `pith`, `terse`, `lac` (laconic). The pun
   (*comm* = communication; visually "less than C") may be worth the collision —
   user decision, tracked as a backlog chunk.
2. `=` vs `==` as canonical equality (Postel accepts both; which to *teach*?).
3. Should `ask` without a shape default to `str`, or require explicitness?
4. Binding via `name:` vs `let` — LL(1) analysis in the spec chunk decides.
5. A `py" … "` escape hatch for inline Python in hosts that allow it — power vs
   security-surface trade; default no.

## 12. What gets built (roadmap pointer)

The native roadmap (think-and-ship; `ROADMAP.md` is the generated view) sequences:
**spec+cost-table → Rust interpreter MVP → agent primitives → cheat sheet →
benchmark (the moment of truth: tokens AND success rate vs Python) → constrained-
decoding artifacts**, with backlog for the MCP server wedge, wasm embedding, naming,
editor tooling, and a live-token-meter playground.

---
*Design grounded in: provider pricing pages & 2026 cost analyses (Redis, Kong,
digitalapplied, iternal); Wang et al., "Executable Code Actions Elicit Better LLM
Agents" (ICML 2024); TOON (toonformat.dev) + arXiv 2603.03306; Alderson,
"Which programming languages are most token-efficient?" (Jan 2026); Ronacher,
"A Language For Agents" (Feb 2026); llguidance/XGrammar/JSONSchemaBench. Token
counts measured with tiktoken o200k_base on 2026-06-10.*
