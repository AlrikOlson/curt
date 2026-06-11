#!/usr/bin/env python3
"""Mechanical grader for the cheatsheet experiment.

Usage: grade.py <answers_dir> [--json]

<answers_dir> holds one `NN_name.curt` per task (same stems as tasks/).
Per task, three cumulative points, all decided by the toolchain:

  parse  — `curt parse` exit 0          (syntax validity)
  check  — `curt check` exit 0          (type validity)
  run    — `curt run` stdout == frozen .expected  (semantic correctness)

A missing answer file scores 0. Never masks an exit code: the grade IS
the exit code.
"""

import json
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CURT = ROOT / "target" / "release" / "curt"
TASKS = sorted(HERE.glob("tasks/*.curt"))


def tool(cmd: str, path: pathlib.Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(CURT), cmd, str(path)], capture_output=True, text=True, timeout=30
    )


def grade_one(answer: pathlib.Path, expected: pathlib.Path) -> dict:
    res = {"parse": False, "check": False, "run": False, "detail": ""}
    if not answer.exists():
        res["detail"] = "missing answer file"
        return res
    p = tool("parse", answer)
    if p.returncode != 0:
        res["detail"] = (p.stdout + p.stderr).strip().splitlines()[-1][:160]
        return res
    res["parse"] = True
    c = tool("check", answer)
    res["check"] = c.returncode == 0
    if not res["check"]:
        res["detail"] = (c.stdout + c.stderr).strip().splitlines()[-1][:160]
    # run is graded independently of check: the evaluator is the semantic
    # oracle, and check-vs-run disagreements are design feedback we want
    # to see in the matrix, not mask.
    r = tool("run", answer)
    want = expected.read_text()
    if r.returncode == 0 and r.stdout == want:
        res["run"] = True
    else:
        res["detail"] = (res["detail"] + f" | run exit {r.returncode}; got {r.stdout!r}")[:160].lstrip(" |")
    return res


def main() -> None:
    answers = pathlib.Path(sys.argv[1])
    rows, totals = [], {"parse": 0, "check": 0, "run": 0}
    for ref in TASKS:
        stem = ref.stem
        r = grade_one(answers / f"{stem}.curt", ref.with_suffix(".expected"))
        rows.append({"task": stem, **r})
        for k in totals:
            totals[k] += r[k]
    n = len(TASKS)
    report = {"answers_dir": str(answers), "n_tasks": n, "totals": totals, "rows": rows}
    if "--json" in sys.argv:
        print(json.dumps(report, indent=2))
    else:
        for r in rows:
            marks = "".join("✓" if r[k] else "✗" for k in ("parse", "check", "run"))
            print(f"{r['task']:<18} {marks}  {r['detail']}")
        print(f"TOTAL parse {totals['parse']}/{n} · check {totals['check']}/{n} · run {totals['run']}/{n}")


if __name__ == "__main__":
    main()
