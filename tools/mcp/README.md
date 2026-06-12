# curt-mcp

MCP code-execution server for [curt](https://curtlang.com) — the
machine-first language for AI agents, where output-token cost is the prime
design directive.

The pattern is [code execution with
MCP](https://www.anthropic.com/engineering/code-execution-with-mcp): instead
of juggling many tool definitions, an agent writes a small curt program and
runs it. curt's whole design (median 1.10× Python token cost on the
measured corpus, single-line JSON diagnostics built for one-turn
self-repair) exists for that loop's economics.

## Tools

- **`run_curt`** — execute a curt program in a sandboxed subprocess (fresh
  temp dir, scrubbed env, wall-clock timeout ≤10s, 64 KiB program cap).
  Capabilities are **denied by default**: `fs`/`net` only with
  `allow_fs`/`allow_net`. Inline `files` are materialized into the sandbox.
  Failures return one JSON diagnostic; when it carries
  `repair.replacement`, the payload is a *verified* whole-line fix — apply
  it verbatim and rerun.
- **`lint_curt`** — provably-equivalent cheaper idioms, same diagnostic
  shape, verified payloads.

## Resources

- `curt://sheet` — the full cheat sheet with twenty execution-verified
  worked examples (~4.6k o200k tokens). Read once per session; it is sized
  past provider cache floors on purpose (measured: cached-large beats
  uncached-small on both cost and success).
- `curt://sheet-lite` — the ~400-token core.

Tool descriptions stay under 500 o200k tokens by design — the language is
taught by the cacheable resource, not the tool schema. Measured
(demo_selfrepair.py asserts these): `run_curt` 225 o200k, `lint_curt` 43;
the `curt://sheet` resource is ~4.6k o200k, once per session, cacheable.

## Run

```bash
# from the curt repo (needs target/release/curt; set CURT_BIN to override)
cd tools/mcp && python3 -m venv .venv && .venv/bin/pip install -e .
.venv/bin/curt-mcp     # stdio transport
```

Claude Desktop / any MCP client config:

```json
{"mcpServers": {"curt": {"command": "/path/to/cmm/tools/mcp/.venv/bin/curt-mcp"}}}
```

## Security

See [THREATMODEL.md](THREATMODEL.md). Short version: curt's interpreter
denies fs/net unless granted per call (that gate is the primary boundary);
the subprocess adds hygiene (temp cwd, env scrub, timeout) but is not an OS
jail. Tool descriptions are static strings in this repo — reviewable, no
dynamic content, no description-injection surface.
