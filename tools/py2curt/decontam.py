#!/usr/bin/env python3
"""Decontamination scan: training pairs vs the frozen evaluation suites.

Compares every (instruction, curt) pair in the given corpus files against
all evaluation material — bench/dbench task prompts, reference task
programs, and frozen expected outputs — using token-shingle Jaccard
similarity. Any pair at or above the threshold is a contamination hit and
the scan exits non-zero.

Both texts are normalized first (lowercase, identifiers and punctuation
kept, numbers folded to `N`, string literals folded to `S`) so renamed
variables and changed constants cannot hide an otherwise-copied task.

Usage:
  .ci-venv/bin/python tools/py2curt/decontam.py data/py2curt/pairs-real.jsonl.gz [...]
"""

import gzip
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SHINGLE = 6
THRESHOLD = 0.5

EVAL_GLOBS = [
    ("bench prompts", "tools/bench/PROMPTS.md"),
    ("bench tasks", "tools/bench/tasks/*"),
    ("dbench tasks", "tools/dbench/tasks/*"),
    ("dbench doc", "tools/dbench/DOMAINBENCH.md"),
]


def norm_tokens(text):
    text = re.sub(r'"[^"\n]*"', " S ", text.lower())
    text = re.sub(r"\d+(\.\d+)?", " N ", text)
    return re.findall(r"[a-z_]+|[^\sa-z_]", text)


def shingles(text):
    toks = norm_tokens(text)
    return {tuple(toks[i:i + SHINGLE]) for i in range(len(toks) - SHINGLE + 1)}


def jaccard(a, b):
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def eval_corpus():
    docs = []
    for label, pat in EVAL_GLOBS:
        for p in sorted(ROOT.glob(pat)):
            if p.is_file():
                docs.append((f"{label}:{p.name}", shingles(p.read_text(errors="ignore"))))
    return docs


def main(paths):
    docs = eval_corpus()
    print(f"eval-side documents: {len(docs)}  (shingle={SHINGLE}, threshold={THRESHOLD})")
    worst = (0.0, None, None)
    hits = []
    total = 0
    for path in paths:
        opener = gzip.open if path.endswith(".gz") else open
        with opener(path, "rt") as f:
            for line in f:
                row = json.loads(line)
                total += 1
                sh = shingles(row["instruction"] + "\n" + row["curt"])
                for name, dsh in docs:
                    j = jaccard(sh, dsh)
                    if j > worst[0]:
                        worst = (j, row["id"], name)
                    if j >= THRESHOLD:
                        hits.append((row["id"], name, j))
    print(f"pairs scanned: {total}")
    print(f"max similarity: {worst[0]:.3f}  ({worst[1]} vs {worst[2]})")
    if hits:
        print(f"CONTAMINATION: {len(hits)} pair(s) at/above threshold:")
        for pid, name, j in hits:
            print(f"  {pid} vs {name}: {j:.3f}")
        return 1
    print("clean: no pair reaches the threshold")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:] or ["data/py2curt/pairs-real.jsonl.gz"]))
