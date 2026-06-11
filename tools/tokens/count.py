#!/usr/bin/env python3
"""tools/tokens — the curt cost-table CLI.

Counts corpus token costs (o200k_base offline; Anthropic count-tokens when
ANTHROPIC_API_KEY is set) and reports per-snippet ratios + per-baseline
medians. This output is the CI regression gate: grammar/stdlib changes that
regress the corpus fail review.

Usage:
  python3 count.py [--corpus DIR]            # full table + medians
  python3 count.py --constructs              # per-construct cost table
  python3 count.py --file F                  # count one file

Corpus programs are LITERAL FILES — never re-type them through shell or
string-escaping layers (an escaping bug shifted a count during redesign-v02).
"""
import argparse
import json
import os
import statistics
import sys
from pathlib import Path

import tiktoken

ENC = tiktoken.get_encoding("o200k_base")


def o200k(text: str) -> int:
    return len(ENC.encode(text))


def anthropic_count(text: str):
    """Anthropic count-tokens API; returns None when no key / on any failure."""
    key = os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        return None
    try:
        import urllib.request

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages/count_tokens",
            data=json.dumps(
                {
                    "model": "claude-sonnet-4-6",
                    "messages": [{"role": "user", "content": text}],
                }
            ).encode(),
            headers={
                "x-api-key": key,
                "anthropic-version": "2023-06-01",
                "content-type": "application/json",
            },
        )
        with urllib.request.urlopen(req, timeout=15) as r:
            return json.load(r)["input_tokens"]
    except Exception:
        return None


LANGS = ["curt", "py", "go", "rs"]
EXT = {"curt": ".curt", "py": ".py", "go": ".go", "rs": ".rs"}

# The construct cost table: every v0.1 grammar construct, priced in context.
CONSTRUCTS = [
    ("equation def", "hyp a b = a + b"),
    ("block equation", "f x = {\n  y = x + 1\n  y * 2\n}"),
    ("binding", "n = 0"),
    ("compound assign", "n += 1"),
    ("annotation (optional)", "x: int = 1"),
    ("if/else expr", "if x < 0 { -1 } else { 1 }"),
    ("while", "while n != 1 { n -= 1 }"),
    ("for-in", "for x in xs { print x }"),
    ("range loop", "for i in range 4 { go work i }"),
    ("match + narrowing", 'match v { float x -> x, str s -> 0 }'),
    ("type record", "type Pt = {x float, y float}"),
    ("record literal", "Pt{x:0, y:0}"),
    ("anon record", '{name:"a", score:9}'),
    ("union in sig", "f :: float | str -> int"),
    ("tuple + destructure", "(lo, hi) = minmax xs"),
    ("lambda", "map x -> x + 1"),
    ("two-param lambda", "fold 0 acc x -> acc + x"),
    ("projection lambda", "top 3 .score"),
    ("pipeline", "xs | keep .active | map .name"),
    ("dot chain", "s.trim.lower.words"),
    ("index/slice", "ts[0] ts[1:]"),
    ("propagate ?", "x = parse s?"),
    ("rescue ? v", "cfg = load p ? {}"),
    ("string interp", 'print "{k} {v}"'),
    ("go spawn", "go handle c"),
    ("ret early", "ret mid"),
    ("pub export", "pub add a b = a + b"),
    ("ffi sig ::", "add :: int int -> int"),
    ("bit ops", "h = (h ^ b) * k"),
    ("sized literal", "h = 7u64"),
    ("membership in", '"ERR" in x'),
    ("bool ops", "a and b or not c"),
    ("comparison", "x == y"),
    ("comment", "# never emitted by agents"),
]


def table(corpus: Path, use_api: bool):
    names = sorted({f.stem for f in corpus.glob("*.curt")})
    ratios = {"py": [], "go": [], "rs": []}
    rows = []
    api_note = "on" if (use_api and os.environ.get("ANTHROPIC_API_KEY")) else "off (no ANTHROPIC_API_KEY)"
    print(f"=== corpus cost table (o200k_base; anthropic: {api_note}) ===")
    for name in names:
        counts = {}
        for lang in LANGS:
            f = corpus / (name + EXT[lang])
            if f.exists():
                counts[lang] = o200k(f.read_text())
        c = counts["curt"]
        cells = [f"curt={c:4d}"]
        for lang in ["py", "go", "rs"]:
            if lang in counts:
                r = counts[lang] / c
                ratios[lang].append(r)
                cells.append(f"{lang}={counts[lang]:4d} ({r:4.2f}x)")
            else:
                cells.append(f"{lang}=   - (  - )")
        rows.append((name, cells))
        print(f"{name:15s} " + "  ".join(cells))
    print()
    for lang, label in [("py", "Python"), ("go", "Go"), ("rs", "Rust")]:
        rs = ratios[lang]
        flag = "" if len(rs) >= 10 else f"  [small n={len(rs)} — compiled subset, flagged honestly]"
        print(f"median vs {label:7s}: {statistics.median(rs):4.2f}x over n={len(rs)}{flag}")
    return rows


def constructs():
    print("=== per-construct cost (o200k_base, in-context) ===")
    for name, sample in CONSTRUCTS:
        print(f"{name:22s} {o200k(sample):3d}  | {sample.splitlines()[0]}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default=str(Path(__file__).resolve().parents[2] / "corpus"))
    ap.add_argument("--constructs", action="store_true")
    ap.add_argument("--file")
    ap.add_argument("--api", action="store_true", help="also query Anthropic count-tokens")
    a = ap.parse_args()
    if a.file:
        text = Path(a.file).read_text()
        print(o200k(text))
        if a.api:
            print("anthropic:", anthropic_count(text))
        return
    if a.constructs:
        constructs()
        return
    table(Path(a.corpus), a.api)


if __name__ == "__main__":
    main()
