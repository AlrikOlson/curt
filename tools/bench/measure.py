#!/usr/bin/env python3
"""Token measurement + ratio summary for token-bench (o200k_base).

Reads grade_bench.py --all --json results (regenerated live), counts
o200k tokens for every answer file, and reports per-language medians on
SOLVED cells plus per-task cross-language ratios where both sides have
at least one solved sample. Also prints the input-side re-read cost for
the multi-file task (14_rect_lib): tokens of the SOLVED solutions —
the cost of holding the module in context for later edits.

Usage: measure.py [--json]
"""

import json
import pathlib
import statistics
import subprocess
import sys

import tiktoken

HERE = pathlib.Path(__file__).resolve().parent
ENC = tiktoken.get_encoding("o200k_base")
LANGS = ["curt", "python", "go", "rust"]
REREAD_TASK = "14_rect_lib"


def main() -> None:
    graded = json.loads(
        subprocess.run(
            [sys.executable, str(HERE / "grade_bench.py"), "--all", "--json"],
            capture_output=True, text=True, check=True,
        ).stdout
    )
    # tokens[lang][task] = list of token counts of SOLVED samples (any model)
    tokens: dict = {l: {} for l in LANGS}
    for rep in graded:
        lane = pathlib.Path(rep["dir"]).parts[1]  # answers/<lang>_<model>/s<k>
        lang = lane.rsplit("_", 1)[0]
        if lang not in LANGS:  # reference lanes (refs_*) are not model cells
            continue
        d = HERE / "answers" / pathlib.Path(rep["dir"]).parts[1] / pathlib.Path(rep["dir"]).parts[2]
        for row in rep["rows"]:
            if not row["ok"]:
                continue
            files = list(d.glob(f"{row['task']}.*"))
            if files:
                n = len(ENC.encode(files[0].read_text()))
                tokens[lang].setdefault(row["task"], []).append(n)

    summary = {"per_lang_median_tokens": {}, "ratios_vs_curt": {}, "reread": {}}
    for lang in LANGS:
        allv = [v for vs in tokens[lang].values() for v in vs]
        summary["per_lang_median_tokens"][lang] = statistics.median(allv) if allv else None
    for lang in LANGS[1:]:
        per_task = []
        for task in tokens["curt"]:
            if task in tokens[lang]:
                per_task.append(
                    statistics.median(tokens[lang][task]) / statistics.median(tokens["curt"][task])
                )
        summary["ratios_vs_curt"][lang] = {
            "median": round(statistics.median(per_task), 2) if per_task else None,
            "min": round(min(per_task), 2) if per_task else None,
            "max": round(max(per_task), 2) if per_task else None,
            "n_tasks": len(per_task),
        }
    for lang in LANGS:
        vs = tokens[lang].get(REREAD_TASK, [])
        summary["reread"][lang] = statistics.median(vs) if vs else None

    if "--json" in sys.argv:
        print(json.dumps(summary, indent=2))
        return
    print("median o200k tokens over solved cells:")
    for lang, v in summary["per_lang_median_tokens"].items():
        print(f"  {lang:<8} {v}")
    print("per-task median ratios vs curt (solved-on-both-sides):")
    for lang, r in summary["ratios_vs_curt"].items():
        print(f"  {lang:<8} median {r['median']}x  range {r['min']}–{r['max']}x  n={r['n_tasks']}")
    print(f"re-read cost ({REREAD_TASK}), median tokens of solved solutions:")
    for lang, v in summary["reread"].items():
        print(f"  {lang:<8} {v}")


if __name__ == "__main__":
    main()
