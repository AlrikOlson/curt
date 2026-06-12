"""curt-mcp — the run_curt code-execution MCP server.

The wedge (chunk:mcp-server, think:171): agents write curt programs to do
work instead of juggling N×M tool definitions — the code-execution-with-MCP
pattern at curt's token economics. One tool runs programs; diagnostics come
back as the same single-line JSON the toolchain emits everywhere, including
verified `repair.replacement` payloads an agent can apply verbatim.

Sandbox stance (THREATMODEL.md has the full statement): each call runs the
curt binary in a fresh temp directory with a wall-clock timeout and a
scrubbed environment. curt's interpreter itself denies fs/net unless the
call grants them — that interpreter-level capability gate is the primary
boundary; the subprocess hygiene is defense in depth, not an OS jail.
"""

import json
import os
import pathlib
import shutil
import subprocess
import tempfile

from mcp.server.fastmcp import FastMCP

ROOT = pathlib.Path(__file__).resolve().parents[3]
CURT = pathlib.Path(os.environ.get("CURT_BIN", ROOT / "target" / "release" / "curt"))
SHEET_LITE = ROOT / "tools" / "bench" / "sheets" / "lite.md"
SHEET_EXT = ROOT / "tools" / "bench" / "sheets" / "ext.md"
MAX_TIMEOUT = 10.0
MAX_PROGRAM_BYTES = 64 * 1024

# Lite by design (≤500 o200k, ci-gated): the model learns curt from the
# curt://sheet resource (cacheable), not from this description.
RUN_DESC = """Run a curt program and return its stdout.

curt is a machine-first language optimized for output-token cost. Quick core:
`x = 1` bind; `f a b = a + b` equation; pipelines `xs | keep (x -> x > 0) | sum`;
`print "{x}"` interpolation; `match v { err e -> .., int n -> .., _ -> .. }`;
`fs.read p ? fallback` rescue, `(fs.read p)?` propagate; `range a b step`.
Read the curt://sheet resource for the full cheat sheet with worked examples.

On failure you get one JSON diagnostic line: {"err","at","want"/"got"/"msg",
"repair":{"id","summary","replacement":[{"line","new"}]?}}. When
`repair.replacement` is present it is a VERIFIED whole-line fix — apply it
verbatim (1-based line numbers) and rerun. Capabilities: fs and net are DENIED
unless you set allow_fs/allow_net; files land in a fresh temp dir per call."""

LINT_DESC = """Lint a curt program for provably-equivalent cheaper idioms.
Returns one JSON finding per line (same shape as run_curt diagnostics);
`repair.replacement` payloads are verified equivalent — safe to apply."""

mcp = FastMCP("curt")


def _scrub_env() -> dict:
    # minimal env: no inherited secrets/proxies; curt needs nothing
    return {"PATH": "/usr/bin:/bin"}


def _run(cmd: list[str], cwd: pathlib.Path, stdin: str | None, timeout: float):
    return subprocess.run(
        cmd,
        cwd=cwd,
        input=stdin,
        capture_output=True,
        text=True,
        timeout=timeout,
        env=_scrub_env(),
    )


@mcp.tool(description=RUN_DESC)
def run_curt(
    program: str,
    stdin: str = "",
    args: list[str] | None = None,
    files: dict[str, str] | None = None,
    allow_fs: bool = False,
    allow_net: bool = False,
    timeout_s: float = 5.0,
) -> dict:
    """Execute `program`. `files` maps relative names to contents, materialized
    in the sandbox dir (implies the program may read them with allow_fs)."""
    if len(program.encode()) > MAX_PROGRAM_BYTES:
        return {"ok": False, "error": "program too large (64 KiB cap)"}
    timeout = max(0.1, min(float(timeout_s), MAX_TIMEOUT))
    if not CURT.exists():
        return {"ok": False, "error": f"curt binary not found at {CURT}"}
    # resolve() the sandbox itself: macOS tempdirs live behind the
    # /var -> /private/var symlink and the escape check compares canonically
    sandbox = pathlib.Path(tempfile.mkdtemp(prefix="curt-mcp-")).resolve()
    try:
        prog = sandbox / "main.curt"
        prog.write_text(program)
        for name, content in (files or {}).items():
            p = (sandbox / name).resolve()
            if not p.is_relative_to(sandbox):
                return {"ok": False, "error": f"file path escapes sandbox: {name}"}
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(content)
        # check first: static diagnostics carry verified repair payloads
        try:
            c = _run([str(CURT), "check", str(prog)], sandbox, None, timeout)
        except subprocess.TimeoutExpired:
            return {"ok": False, "error": f"check timed out after {timeout}s"}
        if c.returncode != 0:
            return _diag_result(c.stderr)
        run_cmd = [str(CURT), "run"]
        if allow_fs:
            run_cmd.append("--fs")
        if allow_net:
            run_cmd.append("--net")
        run_cmd.append(str(prog))
        run_cmd.extend(args or [])
        try:
            r = _run(run_cmd, sandbox, stdin or None, timeout)
        except subprocess.TimeoutExpired:
            return {"ok": False, "error": f"run timed out after {timeout}s"}
        if r.returncode != 0:
            return _diag_result(r.stderr)
        return {"ok": True, "stdout": r.stdout}
    finally:
        shutil.rmtree(sandbox, ignore_errors=True)


@mcp.tool(description=LINT_DESC)
def lint_curt(program: str) -> dict:
    if not CURT.exists():
        return {"ok": False, "error": f"curt binary not found at {CURT}"}
    sandbox = pathlib.Path(tempfile.mkdtemp(prefix="curt-mcp-"))
    try:
        prog = sandbox / "main.curt"
        prog.write_text(program)
        try:
            p = _run([str(CURT), "lint", str(prog)], sandbox, None, MAX_TIMEOUT)
        except subprocess.TimeoutExpired:
            return {"ok": False, "error": "lint timed out"}
        if p.returncode != 0:
            return _diag_result(p.stderr)
        findings = [json.loads(ln) for ln in p.stdout.splitlines() if ln.strip()]
        return {"ok": True, "findings": findings}
    finally:
        shutil.rmtree(sandbox, ignore_errors=True)


def _diag_result(stderr: str) -> dict:
    line = stderr.strip().splitlines()[-1] if stderr.strip() else ""
    try:
        diag = json.loads(line)
    except json.JSONDecodeError:
        return {"ok": False, "error": line or "unknown failure"}
    return {"ok": False, "diagnostic": diag}


@mcp.resource("curt://sheet", description="The full curt cheat sheet with twenty execution-verified worked examples (~4.6k o200k tokens; cacheable — read once per session).", mime_type="text/markdown")
def sheet() -> str:
    return SHEET_EXT.read_text()


@mcp.resource("curt://sheet-lite", description="Minimal curt core (~400 o200k tokens).", mime_type="text/markdown")
def sheet_lite() -> str:
    return SHEET_LITE.read_text()


def main() -> None:
    mcp.run()


if __name__ == "__main__":
    main()
