#!/usr/bin/env python3
"""Ceremony fraction (hypothesis hx4, think:141): a static, model-free
instrument for a language's agent output economy.

FROZEN DEFINITION (locked in the reasoning trace before computation):
for a language's solved programs P1..PN over N DISTINCT tasks, tokenize
with o200k_base; df(t) = fraction of programs containing token type t;
ceremony tokens = occurrences of types with df > 0.5;
CF = ceremony occurrences / total occurrences.

Program sets ($0, committed artifacts only):
  curt / python / zero — first solved sample per task, sonnet h2h lanes
                          (27/27 each; 9 programs per language)
  go / rust            — corpus parallel translations (6 tasks)
  curt6 / py6          — same 6 corpus tasks (set-sensitivity check)

FROZEN PREDICTION: CF(zero) > CF(go) >= CF(rust) > CF(python) > CF(curt);
on the 3 lane-measured languages, CF rank matches median-out-tok rank
(zero 168 >> python 40 >= curt 35; near-tie allowed for python/curt).
REFUTATION: CF(zero) <= CF(python), or CF(curt) > CF(python) + 5pp.
"""

import json
import pathlib
import sys

import tiktoken

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ENC = tiktoken.get_encoding("o200k_base")
CORPUS6 = ["04_binsearch", "07_errors", "10_group", "18_wordfreq", "19_parser", "20_server"]
MEASURED = {"curt": 35, "zero": 168, "py": 40}  # sonnet h2h median out-tok


def h2h_programs(lang: str) -> list[str]:
    """First solved sample per task from the sonnet h2h lane."""
    import re
    lane = HERE / "h2h" / f"sonnet_{lang}.jsonl"
    progs: dict[str, str] = {}
    for ln in lane.open():
        c = json.loads(ln)
        if c["solved"] and c["task"] not in progs:
            reply = c["turns"][-1]["reply"]
            m = re.search(r"```[a-z0-9]*\n?(.*?)```", reply, re.DOTALL)
            progs[c["task"]] = (m.group(1) if m else reply).strip()
    return list(progs.values())


def corpus_programs(ext: str) -> list[str]:
    return [(ROOT / "corpus" / f"{t}.{ext}").read_text() for t in CORPUS6]


def cf(programs: list[str]) -> tuple[float, int, list[str]]:
    """Ceremony fraction, total occurrences, top ceremony token strings."""
    n = len(programs)
    toks = [ENC.encode(p) for p in programs]
    df: dict[int, int] = {}
    for ts in toks:
        for t in set(ts):
            df[t] = df.get(t, 0) + 1
    ceremony_types = {t for t, d in df.items() if d / n > 0.5}
    total = sum(len(ts) for ts in toks)
    cer = sum(1 for ts in toks for t in ts if t in ceremony_types)
    top = sorted(ceremony_types,
                 key=lambda t: -sum(ts.count(t) for ts in toks))[:12]
    return cer / total, total, [ENC.decode([t]) for t in top]


def main() -> int:
    sets = {
        "curt": h2h_programs("curt"),
        "zero": h2h_programs("zero"),
        "py": h2h_programs("py"),
        "go": corpus_programs("go"),
        "rust": corpus_programs("rs"),
        "curt6": corpus_programs("curt"),
        "py6": corpus_programs("py"),
    }
    print(f"{'set':7s}{'n':>3s}{'CF':>8s}{'tokens':>8s}  measured-med-out  top ceremony tokens")
    rows = {}
    for name, progs in sets.items():
        f, total, top = cf(progs)
        rows[name] = f
        meas = MEASURED.get(name, "")
        print(f"{name:7s}{len(progs):>3d}{f:8.3f}{total:>8d}  {str(meas):>16s}  {' '.join(repr(t) for t in top[:8])}")
    # verdict against the frozen prediction
    p1 = rows["zero"] > rows["go"] >= rows["rust"] > rows["py"] > rows["curt"]
    refute = rows["zero"] <= rows["py"] or rows["curt"] > rows["py"] + 0.05
    print(f"\nfrozen ordering zero>go>=rust>py>curt: {'HOLDS' if p1 else 'BROKEN'}")
    print(f"refutation condition triggered: {'YES' if refute else 'no'}")
    print("lane-measured rank consistency (zero >> py >= curt):",
          "HOLDS" if rows["zero"] > rows["py"] and rows["zero"] > rows["curt"] else "BROKEN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
