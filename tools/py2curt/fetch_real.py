#!/usr/bin/env python3
"""Fetch the real-Python sources into data/external/ (gitignored).

Sources (all redistributable; see REPORT-real.md for licenses):
  mbpp.jsonl          google-research MBPP, 974 problems (CC BY 4.0)
  HumanEval.jsonl.gz  OpenAI HumanEval, 164 problems (MIT)
  mbppplus.jsonl      EvalPlus MBPP+ task ids (Apache-2.0) — used as a
                      hand-verified membership flag over MBPP, converted
                      from the HF parquet to jsonl for a pyarrow-free read

Idempotent: skips files that already exist. One command, no arguments.
"""

import json
import pathlib
import subprocess
import sys
import urllib.request

EXT = pathlib.Path(__file__).resolve().parents[2] / "data" / "external"

MBPP_URL = "https://raw.githubusercontent.com/google-research/google-research/master/mbpp/mbpp.jsonl"
HE_URL = "https://raw.githubusercontent.com/openai/human-eval/master/data/HumanEval.jsonl.gz"
MBPPPLUS_REPO = "evalplus/mbppplus"


def fetch(url, dest):
    if dest.exists():
        return
    print(f"fetching {dest.name} ...", file=sys.stderr)
    with urllib.request.urlopen(url) as r:
        dest.write_bytes(r.read())


def fetch_mbppplus(dest):
    if dest.exists():
        return
    print("fetching mbppplus (HF) ...", file=sys.stderr)
    tmp = EXT / "_mbppplus_hf"
    subprocess.run(
        ["hf", "download", MBPPPLUS_REPO, "--repo-type", "dataset", "--local-dir", str(tmp)],
        check=True, capture_output=True, text=True,
    )
    import pyarrow.parquet as pq

    table = pq.read_table(next((tmp / "data").glob("*.parquet")))
    rows = [
        {c: table.column(c)[i].as_py() for c in ("task_id", "prompt", "code", "test_list")}
        for i in range(table.num_rows)
    ]
    with dest.open("w") as f:
        for row in rows:
            f.write(json.dumps(row, sort_keys=True) + "\n")


def main():
    EXT.mkdir(parents=True, exist_ok=True)
    fetch(MBPP_URL, EXT / "mbpp.jsonl")
    fetch(HE_URL, EXT / "HumanEval.jsonl.gz")
    fetch_mbppplus(EXT / "mbppplus.jsonl")
    for name in ("mbpp.jsonl", "HumanEval.jsonl.gz", "mbppplus.jsonl"):
        p = EXT / name
        print(f"{name}: {p.stat().st_size} bytes")


if __name__ == "__main__":
    main()
