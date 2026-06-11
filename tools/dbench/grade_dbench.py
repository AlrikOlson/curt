#!/usr/bin/env python3
"""Mechanical grader for domain-bench (ceremony domains).

Same contract as tools/bench/grade_bench.py with two differences:
all programs run with cwd = tools/dbench/fixtures (the shared file
fixtures), and curt programs get `--fs` (the capability the tasks need;
Python/Go/Rust have ambient fs access — the asymmetry is curt's
sandbox-first model, noted in the report).

Usage: grade_dbench.py <answers_dir>|--all [--json]
"""

import json
import pathlib
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CURT = ROOT / "target" / "release" / "curt"
FIXTURES = HERE / "fixtures"
TASKS = sorted(p for p in (HERE / "tasks").glob("*.expected"))
STEMS = [p.stem for p in TASKS]


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


def run(cmd, timeout=60):
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, cwd=FIXTURES)


def execute(path: pathlib.Path):
    ext = path.suffix.lstrip(".")
    try:
        if ext == "curt":
            return run([str(CURT), "run", "--fs", str(path)])
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
        r = execute(cands[0])
        want = (HERE / "tasks" / f"{stem}.expected").read_text()
        if r is None:
            row["detail"] = "timeout/unsupported"
        elif r.returncode != 0:
            err = (r.stdout + r.stderr).strip()
            row["detail"] = (err.splitlines()[-1][:160] if err else f"exit {r.returncode}")
        elif not norm_eq(r.stdout, want):
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
        print(f"{rep['dir']:<30} {rep['solved']:>2}/{rep['n']}  {marks}")
        for r in rep["rows"]:
            if not r["ok"]:
                print(f"    {r['task']:<20} {r['detail']}")


if __name__ == "__main__":
    main()
