# curt-mcp threat model

Scoped to the v1 subprocess server (chunk:mcp-server, 2026-06-12). Claims
here are meant to be precise; where the boundary is weak we say so.

## What an attacker controls

A malicious or confused agent (or a prompt-injected upstream) controls the
`program`, `stdin`, `args`, `files`, and flag arguments of `run_curt` /
`lint_curt` calls. Nothing else: this server takes no URLs, no dynamic tool
registration, no outbound calls of its own.

## Boundaries, strongest first

1. **The interpreter's capability gate (primary).** curt denies `fs` and
   `net` at the language level unless the call passes `allow_fs`/`allow_net`
   — a denied op yields a rescuable `err` value, never ambient access. This
   is enforced inside the curt binary (eval.rs `Caps`), not by the Python
   wrapper, so a sandbox-dir escape via curt code is not possible without
   the flag. With `allow_fs`, fs access is still relative to the fresh
   sandbox cwd — but absolute paths are NOT blocked by the interpreter;
   treat `allow_fs: true` as trusting the program with user-level file
   access. This is the honest weak edge; grant it only for programs you'd
   run by hand.
2. **Per-call subprocess hygiene (defense in depth, not a jail).** Fresh
   `mkdtemp` cwd deleted after the call; environment scrubbed to a bare
   PATH (no inherited secrets/tokens/proxies); wall-clock timeout (≤10s,
   default 5) kills runaway programs; 64 KiB program cap; inline `files`
   are canonical-path-checked against the sandbox root (symlink-resolved on
   both sides).
3. **Static tool descriptions.** Both descriptions are constant strings in
   this repository — reviewable, version-controlled, no runtime
   composition. The 2026 description-injection attack class (instructions
   smuggled into dynamically generated descriptions) has no surface here.
   The `curt://sheet` resource is likewise a committed file.

## What this server does NOT defend against

- A kernel- or runtime-level escape from the curt binary itself (it is a
  Rust interpreter, not a hypervisor). There is no seccomp/sandbox-exec
  layer in v1; if you need OS-level isolation, run the whole server in a
  container/VM.
- Resource exhaustion below the timeout (a 5-second CPU burn is allowed by
  design).
- A hostile MCP *client* — transport security and client auth are the
  host's concern (stdio server; no network listener of its own).

## Known CVE-class precedents motivating the design

- Tool-description injection (state-of-MCP-security reports, 2026-03;
  NSA CSI on MCP): mitigated by static descriptions.
- Confused-deputy / token-passthrough server patterns: not applicable —
  this server holds no credentials and makes no outbound calls.
