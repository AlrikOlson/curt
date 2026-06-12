#!/usr/bin/env python3
"""Deterministic self-repair demo over real MCP stdio (chunk:mcp-server).

Plays the agent's role without an LLM: submit a broken program via the
run_curt tool, receive the diagnostic with a VERIFIED repair.replacement
payload, apply it mechanically, rerun, and assert success. This is the
one-turn repair loop the payloads were built for (fix-synthesis), exercised
end-to-end through the MCP transport.

Also asserts the token-budget acceptance: each tool description ≤500 o200k.

Exit 0 only if every step holds. Run:
  tools/mcp/.venv/bin/python tools/mcp/demo_selfrepair.py
"""

import asyncio
import json
import pathlib
import sys

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

HERE = pathlib.Path(__file__).resolve().parent
VENV_PY = HERE / ".venv" / "bin" / "python"

BROKEN = 'v = "7"\nprint (match v.int { err _ -> 0, n -> n })\nprint (1 + 2\n'
# line 3 is missing its closing paren: check emits an `expected` diag with a
# verified replacement; lines 1-2 also carry a lint finding (match-rescue)


def apply_replacement(program: str, reps: list[dict]) -> str:
    lines = program.splitlines()
    for r in sorted(reps, key=lambda r: -r["line"]):
        i = r["line"] - 1
        if 0 <= i < len(lines):
            lines[i : i + 1] = r["new"].split("\n")
    return "\n".join(lines) + "\n"


async def main() -> int:
    params = StdioServerParameters(
        command=str(VENV_PY), args=["-m", "curt_mcp.server"], cwd=str(HERE)
    )
    async with stdio_client(params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()

            # 0) token budget: every tool description ≤500 o200k
            tools = await session.list_tools()
            import tiktoken
            enc = tiktoken.get_encoding("o200k_base")
            for t in tools.tools:
                n = len(enc.encode(t.description or ""))
                print(f"tool {t.name}: description {n} o200k")
                assert n <= 500, f"{t.name} description over budget: {n}"

            # 1) the sheet resource is served and substantial
            sheet = await session.read_resource("curt://sheet")
            sheet_text = sheet.contents[0].text
            assert "curt" in sheet_text and len(sheet_text) > 5000
            print(f"resource curt://sheet: {len(sheet_text)} chars")

            # 2) broken program → diagnostic with verified replacement
            r1 = await session.call_tool("run_curt", {"program": BROKEN})
            out1 = json.loads(r1.content[0].text)
            assert not out1["ok"], out1
            diag = out1["diagnostic"]
            reps = diag.get("repair", {}).get("replacement")
            print(f"diagnostic: {diag['err']} at {diag['at']}; replacement: {bool(reps)}")
            assert reps, "expected a verified repair.replacement payload"

            # 3) apply mechanically, rerun → success (the one-turn repair)
            fixed = apply_replacement(BROKEN, reps)
            r2 = await session.call_tool("run_curt", {"program": fixed})
            out2 = json.loads(r2.content[0].text)
            assert out2["ok"], out2
            assert out2["stdout"] == "7\n3\n", out2
            print(f"self-repair OK: stdout {out2['stdout']!r}")

            # 4) lint flows through with its own verified payload
            r3 = await session.call_tool("lint_curt", {"program": fixed})
            out3 = json.loads(r3.content[0].text)
            assert out3["ok"] and out3["findings"], out3
            f0 = out3["findings"][0]
            assert "replacement" in f0["repair"], f0
            print(f"lint finding: {f0['msg']}")

            # 5) caps are denied by default: fs read errs without allow_fs
            r4 = await session.call_tool(
                "run_curt",
                {"program": 'print (fs.read "x.txt" ? "DENIED")\n',
                 "files": {"x.txt": "secret"}},
            )
            out4 = json.loads(r4.content[0].text)
            assert out4["ok"] and out4["stdout"] == "DENIED\n", out4
            r5 = await session.call_tool(
                "run_curt",
                {"program": 'print (fs.read "x.txt" ? "DENIED")\n',
                 "files": {"x.txt": "granted"}, "allow_fs": True},
            )
            out5 = json.loads(r5.content[0].text)
            assert out5["ok"] and out5["stdout"] == "granted\n", out5
            print("capability gate OK: denied by default, granted on allow_fs")

            # 6) timeout kills runaway programs
            r6 = await session.call_tool(
                "run_curt",
                {"program": "n = 1\nwhile n > 0 { n += 1 }\n", "timeout_s": 1},
            )
            out6 = json.loads(r6.content[0].text)
            assert not out6["ok"] and "timed out" in out6.get("error", ""), out6
            print("timeout OK")

    print("ALL DEMO STEPS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
