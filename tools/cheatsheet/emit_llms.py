#!/usr/bin/env python3
"""Emit docs/llms.txt from CHEATSHEET.md (the canonical source).

llms.txt convention (llmstxt.org): H1 title, one-line blockquote summary,
then content. We embed the full cheat sheet inline — it IS the
LLM-facing documentation, sized for direct context inclusion — followed
by links to the deeper artifacts.

Deterministic: same input bytes -> same output bytes (CI diffs it).
"""

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]

HEADER = """\
# curt

> curt is a general-purpose programming language for AI agents, optimized
> for output-token cost (measured, never estimated). The cheat sheet below
> is the complete LLM-facing language reference (~1.5k o200k tokens).

"""

FOOTER = """

## Deeper artifacts

- [SPEC.md](https://github.com/curtlang/curt/blob/main/SPEC.md): full v0.1 specification
- [corpus/](https://github.com/curtlang/curt/tree/main/corpus): 20 measured programs with Python/Go/Rust twins
- [tools/grammar/](https://github.com/curtlang/curt/tree/main/tools/grammar): Lark CFG + GBNF for constrained decoding
- [DESIGN.md](https://github.com/curtlang/curt/blob/main/DESIGN.md): design rationale and measurements
"""


def emit() -> str:
    sheet = (ROOT / "CHEATSHEET.md").read_text()
    # Drop the sheet's own H1 (the llms.txt header provides it).
    body = sheet.split("\n", 1)[1].lstrip("\n")
    return HEADER + body.rstrip("\n") + FOOTER


def main() -> None:
    out = ROOT / "docs" / "llms.txt"
    out.parent.mkdir(parents=True, exist_ok=True)
    text = emit()
    if "--check" in sys.argv:
        if not out.exists() or out.read_text() != text:
            print("docs/llms.txt is stale — run tools/cheatsheet/emit_llms.py", file=sys.stderr)
            sys.exit(1)
        print("docs/llms.txt is current")
        return
    out.write_text(text)
    print(f"wrote {out} ({len(text)} bytes)")


if __name__ == "__main__":
    main()
