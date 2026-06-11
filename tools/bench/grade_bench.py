#!/usr/bin/env python3
"""Mechanical grader for token-bench.

Usage:
  grade_bench.py <answers_dir> [--json]      # one lane-sample dir
  grade_bench.py --all [--json]              # every dir under answers/

A lane-sample dir (e.g. answers/curt_haiku/s1/) holds one solution per
task, named <task_stem>.<ext> with ext in {curt, py, go, rs}. Execution:

  .curt -> target/release/curt run       (parse/check also recorded)
  .py   -> python3
  .go   -> go run
  .rs   -> rustc -O -o tmp && run

Stdout is compared to tasks/<stem>.expected line-by-line; whitespace
tokens that parse as numbers in both are compared numerically (so `86`
== `86.0` across languages); everything else compares exactly.
The grade IS the exit code path — nothing is masked.
"""

import json
import pathlib
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CURT = ROOT / "target" / "release" / "curt"
TASKS = sorted(p for p in (HERE / "tasks").glob("*.expected"))
STEMS = [p.stem for p in TASKS]
EXTS = {"curt": "curt", "python": "py", "go": "go", "rust": "rs"}


def norm_eq(got: str, want: str) -> bool:
    gl, wl = got.rstrip("\n").split("\n"), want.rstrip("\n").split("\n")
    if len(gl) != len(wl):
        return False
    for g, w in zip(gl, wl):
        gt, wt = g.split(), w.split()
        if len(gt) != len(wt):
            return False
        for a, b in zip(gt, wt):
            if a == b:
                continue
            try:
                if abs(float(a) - float(b)) < 1e-9:
                    continue
            except ValueError:
                return False
            else:
                continue
            return False
    return True


def run(cmd, timeout=60) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)


def execute(path: pathlib.Path) -> subprocess.CompletedProcess | None:
    ext = path.suffix.lstrip(".")
    try:
        if ext == "curt":
            return run([str(CURT), "run", str(path)])
        if ext == "py":
            return run(["python3", str(path)])
        if ext == "go":
            return run(["go", "run", str(path)], timeout=120)
        if ext == "rs":
            with tempfile.TemporaryDirectory() as td:
                exe = pathlib.Path(td) / "prog"
                c = run(["rustc", "-O", "-o", str(exe), str(path)], timeout=120)
                if c.returncode != 0:
                    return c
                return run([str(exe)])
    except subprocess.TimeoutExpired:
        return None
    return None


def grade_dir(d: pathlib.Path) -> dict:
    rows, solved = [], 0
    for stem in STEMS:
        cands = list(d.glob(f"{stem}.*"))
        row = {"task": stem, "ok": False, "detail": ""}
        if not cands:
            row["detail"] = "missing"
            rows.append(row)
            continue
        ans = cands[0]
        if ans.suffix == ".curt":
            p = run([str(CURT), "parse", str(ans)])
            c = run([str(CURT), "check", str(ans)])
            row["parse"], row["check"] = p.returncode == 0, c.returncode == 0
        r = execute(ans)
        if r is None:
            row["detail"] = "timeout/unsupported"
        elif r.returncode != 0:
            row["detail"] = (r.stdout + r.stderr).strip().splitlines()[-1][:160] if (r.stdout + r.stderr).strip() else f"exit {r.returncode}"
        elif not norm_eq(r.stdout, (HERE / "tasks" / f"{stem}.expected").read_text()):
            row["detail"] = f"wrong output: {r.stdout!r}"[:160]
        else:
            row["ok"] = True
            solved += 1
        rows.append(row)
    return {"dir": str(d.relative_to(HERE)), "solved": solved, "n": len(STEMS), "rows": rows}


def main() -> None:
    if "--all" in sys.argv:
        dirs = sorted(p for p in (HERE / "answers").glob("*/s*") if p.is_dir())
    else:
        dirs = [pathlib.Path(sys.argv[1]).resolve()]
    reports = [grade_dir(d) for d in dirs]
    if "--json" in sys.argv:
        print(json.dumps(reports, indent=2))
        return
    for rep in reports:
        marks = "".join("✓" if r["ok"] else "✗" for r in rep["rows"])
        print(f"{rep['dir']:<28} {rep['solved']:>2}/{rep['n']}  {marks}")
        for r in rep["rows"]:
            if not r["ok"]:
                print(f"    {r['task']:<16} {r['detail']}")


if __name__ == "__main__":
    main()
