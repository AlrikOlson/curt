#!/usr/bin/env python3
"""Reproduce the context-cost numbers in RESULTS.md (o200k_base)."""

import pathlib

import tiktoken

ROOT = pathlib.Path(__file__).resolve().parents[3]
ENC = tiktoken.get_encoding("o200k_base")


def count(path: pathlib.Path) -> int:
    return len(ENC.encode(path.read_text()))


def main() -> None:
    spec = count(ROOT / "SPEC.md")
    corpus = sum(count(p) for p in sorted((ROOT / "corpus").glob("*.curt")))
    sheet = count(ROOT / "CHEATSHEET.md")
    print(f"SPEC.md                 {spec}")
    print(f"corpus *.curt (20)      {corpus}")
    print(f"arm B total             {spec + corpus}")
    print(f"CHEATSHEET.md (arm C)   {sheet}")
    print(f"B / C ratio             {(spec + corpus) / sheet:.1f}x")


if __name__ == "__main__":
    main()
