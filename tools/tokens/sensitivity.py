#!/usr/bin/env python3
"""Tokenizer sensitivity — replicate every token claim across tokenizers.

Lanes:
  o200k_base, cl100k_base        (tiktoken, local)
  qwen2.5-coder, deepseek-coder  (HF tokenizer.json, local)
  anthropic                      (count-tokens API; runs ONLY if
                                  ANTHROPIC_API_KEY is set — else SKIPPED)

Measures, per tokenizer:
  1. corpus totals + median per-file ratio (python/curt)
  2. bench (15 tasks) + dbench (10 tasks) per-task median ratios on the
     committed solved lanes (curt v2/v3 + dbench curt_*_v2 vs python)
  3. stdlib verb audit: token count of each verb bare / dotted / piped
     (worst case reported; >1 in the dotted form = fragmenting)

Usage: .ci-venv/bin/python tools/tokens/sensitivity.py
"""

import json
import os
import pathlib
import statistics
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]

VERBS = (
    "len map keep fold sum min max sort rev top group counts pairs first last "
    "flat join split words lines chars bytes trim lower upper replace range "
    "int float str json print err"
).split()


def lanes():
    out = {}
    import tiktoken

    for name in ("o200k_base", "cl100k_base"):
        enc = tiktoken.get_encoding(name)
        out[name] = lambda s, e=enc: len(e.encode(s))
    from huggingface_hub import hf_hub_download
    from tokenizers import Tokenizer

    for label, repo in (
        ("qwen2.5-coder", "Qwen/Qwen2.5-Coder-7B-Instruct"),
        ("deepseek-coder", "deepseek-ai/deepseek-coder-6.7b-instruct"),
    ):
        tok = Tokenizer.from_file(hf_hub_download(repo, "tokenizer.json"))
        out[label] = lambda s, t=tok: len(t.encode(s, add_special_tokens=False).ids)
    if os.environ.get("ANTHROPIC_API_KEY"):
        import urllib.request

        def anthropic_count(s):
            req = urllib.request.Request(
                "https://api.anthropic.com/v1/messages/count_tokens",
                data=json.dumps(
                    {"model": "claude-sonnet-4-6", "messages": [{"role": "user", "content": s}]}
                ).encode(),
                headers={
                    "x-api-key": os.environ["ANTHROPIC_API_KEY"],
                    "anthropic-version": "2023-06-01",
                    "content-type": "application/json",
                },
            )
            with urllib.request.urlopen(req) as r:
                return json.load(r)["input_tokens"]

        out["anthropic"] = anthropic_count
    else:
        print("anthropic lane: SKIPPED (set ANTHROPIC_API_KEY to enable)", file=sys.stderr)
    return out


def solved_pairs(suite):
    """(task -> {lang: [token-counted file paths]}) for solved committed cells."""
    if suite == "bench":
        here, grader = ROOT / "tools/bench", ROOT / "tools/bench/grade_bench.py"
        curt_lanes = ("curt_haiku_v2", "curt_sonnet_v2", "curt_haiku_v3", "curt_sonnet_v3")
    else:
        here, grader = ROOT / "tools/dbench", ROOT / "tools/dbench/grade_dbench.py"
        curt_lanes = ("curt_haiku_v2", "curt_sonnet_v2")
    py_lanes = ("python_haiku", "python_sonnet")
    graded = json.loads(
        subprocess.run(
            [sys.executable, str(grader), "--all", "--json"], capture_output=True, text=True, check=True
        ).stdout
    )
    files = {}
    for rep in graded:
        lane = pathlib.Path(rep["dir"]).parts[-2]
        lang = "curt" if lane in curt_lanes else ("python" if lane in py_lanes else None)
        if lang is None:
            continue
        d = here / rep["dir"] if not (here / rep["dir"]).is_absolute() else pathlib.Path(rep["dir"])
        d = here / "answers" / pathlib.Path(rep["dir"]).parts[-2] / pathlib.Path(rep["dir"]).parts[-1]
        for row in rep["rows"]:
            if not row["ok"]:
                continue
            hit = list(d.glob(f"{row['task']}.*"))
            if hit:
                files.setdefault(row["task"], {}).setdefault(lang, []).append(hit[0])
    return files


def per_task_ratio(files, count):
    ratios = []
    for task, langs in files.items():
        if "curt" not in langs or "python" not in langs:
            continue
        c = statistics.median(count(p.read_text()) for p in langs["curt"])
        p = statistics.median(count(q.read_text()) for q in langs["python"])
        ratios.append(p / c)
    return statistics.median(ratios), len(ratios)


def main():
    counts = lanes()
    corpus_curt = sorted((ROOT / "corpus").glob("*.curt"))
    corpus_py = [p.with_suffix(".py") for p in corpus_curt]
    bench = solved_pairs("bench")
    dbench = solved_pairs("dbench")

    print("| tokenizer | corpus py/curt | bench py/curt | dbench py/curt | fragmenting verbs |")
    print("|---|---|---|---|---|")
    for name, count in counts.items():
        pairs = [(count(c.read_text()), count(p.read_text())) for c, p in zip(corpus_curt, corpus_py) if p.exists()]
        corpus_ratio = statistics.median(p / c for c, p in pairs)
        bench_r, bn = per_task_ratio(bench, count)
        dbench_r, dn = per_task_ratio(dbench, count)
        frag = []
        for v in VERBS:
            worst = max(count(v), count(f".{v}") - count("."), count(f"| {v}") - count("| "))
            if worst > 1:
                frag.append(f"{v}({worst})")
        print(
            f"| {name} | {corpus_ratio:.2f}x (n={len(pairs)}) | {bench_r:.2f}x (n={bn}) "
            f"| {dbench_r:.2f}x (n={dn}) | {', '.join(frag) if frag else 'none'} |"
        )


if __name__ == "__main__":
    main()
