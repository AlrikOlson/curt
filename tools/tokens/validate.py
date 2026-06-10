#!/usr/bin/env python3
"""Grammar validation gate: grammar.peg must parse every corpus/*.cmm file.

Exit 0 only on 20/20 (or N/N). This is the 'grammar passes a PEG validity
check' acceptance gate for lang-spec-v01 and a permanent CI gate after.
"""
import sys
from pathlib import Path

from parsimonious.grammar import Grammar
from parsimonious.exceptions import ParseError, IncompleteParseError

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    grammar = Grammar((HERE / "grammar.peg").read_text())
    corpus = sorted((ROOT / "corpus").glob("*.cmm"))
    ok, bad = 0, []
    for f in corpus:
        text = f.read_text()
        if not text.endswith("\n"):
            text += "\n"
        try:
            grammar.parse(text)
            ok += 1
            print(f"  PASS {f.name}")
        except (ParseError, IncompleteParseError) as e:
            bad.append(f.name)
            line = getattr(e, "line", lambda: "?")
            print(f"  FAIL {f.name}: {str(e)[:160]}")
    print(f"\n{ok}/{len(corpus)} corpus files parse")
    sys.exit(0 if not bad else 1)


if __name__ == "__main__":
    main()
