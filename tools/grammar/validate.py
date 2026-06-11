#!/usr/bin/env python3
"""gd-a divergence gate: cmm.lark must (1) parse the golden corpus 20/20 and
(2) AGREE WITH THE RUST PARSER in the reject direction on a negative sample
set. Exit 0 only when both hold.

The Lark twin is kept honest the same way grammar.peg is — by this gate, not
by mechanical derivation (PEG ordered choice is not a CFG transform).

Usage: tools/grammar/validate.py   (run from the repo root; needs `lark` and
a release build of the Rust parser at target/release/cmm)
"""

import pathlib
import subprocess
import sys

import lark

ROOT = pathlib.Path(__file__).resolve().parents[2]
GRAMMAR = pathlib.Path(__file__).resolve().parent / "cmm.lark"
CMM_BIN = ROOT / "target" / "release" / "cmm"

# Invalid snippets: every one must be rejected by BOTH parsers. Chosen to
# probe structure, not lexing trivia.
NEGATIVE = [
    'f = ',                       # binding without a value
    'if {',                       # header brace rule + unclosed block
    'x ->',                       # lambda without a body
    '1 +',                        # dangling operator
    'match x { float',            # unclosed match arm
    '"unterminated',              # bad string literal
    '= 5',                        # no binding target
    'type = {x int}',             # type decl without a name
    'f x = }',                    # stray closing brace
    'pub :: int -> int',          # pub signature without a name
    'print if c { 1 } else { 2 }',  # if-expr as application ARG (head-only)
    'f x match y { _ -> 1 }',       # match-expr as application ARG
]


def lark_accepts(parser: lark.Lark, src: str) -> bool:
    try:
        parser.parse(src)
        return True
    except Exception:
        return False


def rust_accepts(src: str) -> bool:
    proc = subprocess.run(
        [str(CMM_BIN), "parse", "-"], input=src.encode(),
        capture_output=True, check=False,
    )
    return proc.returncode == 0


def main() -> int:
    parser = lark.Lark(GRAMMAR.read_text(), parser="earley", start="start")

    corpus = sorted((ROOT / "corpus").glob("*.cmm"))
    ok = 0
    for path in corpus:
        src = path.read_text()
        if lark_accepts(parser, src):
            ok += 1
        else:
            try:
                parser.parse(src)
            except Exception as exc:  # show the first failure verbosely
                print(f"LARK REJECT {path.name}: {str(exc).splitlines()[0]}")
    print(f"lark corpus: {ok}/{len(corpus)}")

    agree = 0
    for snippet in NEGATIVE:
        l_rej = not lark_accepts(parser, snippet + "\n")
        r_rej = not rust_accepts(snippet + "\n")
        if l_rej and r_rej:
            agree += 1
        else:
            print(f"DISAGREE {snippet!r}: lark_rejects={l_rej} rust_rejects={r_rej}")
    print(f"negative agreement: {agree}/{len(NEGATIVE)}")

    good = ok == len(corpus) == 20 and agree == len(NEGATIVE)
    return 0 if good else 1


if __name__ == "__main__":
    sys.exit(main())
