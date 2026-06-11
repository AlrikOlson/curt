#!/usr/bin/env python3
"""One-command data pipeline: seeds -> verified (instruction, curt) pairs.

Per seed: run the Python source (the oracle), transpile to curt, check,
run, compare stdout (numeric-normalized), canonicalize through
`curt dense` when the program contains a loop, re-verify, and emit JSONL
with full provenance. Rejections carry a taxonomy tag. Deterministic
end-to-end; rerunning reproduces the same pairs byte-for-byte.

Usage:
  .ci-venv/bin/python tools/py2curt/pipeline.py            # full run
  .ci-venv/bin/python tools/py2curt/pipeline.py --golden   # golden suite only
"""

import gzip
import json
import pathlib
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CURT = ROOT / "target" / "release" / "curt"
OUT = ROOT / "data" / "py2curt"

sys.path.insert(0, str(HERE))
from gen_seeds import gen  # noqa: E402
from transpile import Unsupported, transpile  # noqa: E402


def run_py(src):
    r = subprocess.run([sys.executable, "-c", src], capture_output=True, text=True, timeout=10)
    return r.stdout if r.returncode == 0 else None


def run_curt(cmd, src):
    r = subprocess.run([str(CURT), cmd, "-"], input=src, capture_output=True, text=True, timeout=20)
    return r.stdout if r.returncode == 0 else None


def norm_eq(a, b):
    la, lb = a.rstrip("\n").split("\n"), b.rstrip("\n").split("\n")
    if len(la) != len(lb):
        return False
    for x, y in zip(la, lb):
        tx, ty = x.split(), y.split()
        if len(tx) != len(ty):
            return False
        for u, v in zip(tx, ty):
            if u == v:
                continue
            # bool tokens render True/False in Python, true/false in curt
            if u.lower() == v.lower() and u.lower() in ("true", "false"):
                continue
            try:
                if abs(float(u) - float(v)) < 1e-9:
                    continue
            except ValueError:
                return False
            else:
                continue
            return False
    return True


def process(seed):
    expected = run_py(seed["python"])
    if expected is None:
        return {**seed, "status": "reject", "why": "python-failed"}
    try:
        curt_src = transpile(seed["python"])
    except Unsupported as e:
        return {**seed, "status": "reject", "why": f"unsupported:{e.tag}"}
    except SyntaxError:
        return {**seed, "status": "reject", "why": "py-syntax"}
    if run_curt("check", curt_src) is None:
        return {**seed, "status": "reject", "why": "curt-check"}
    got = run_curt("run", curt_src)
    if got is None:
        return {**seed, "status": "reject", "why": "curt-runtime"}
    if not norm_eq(got, expected):
        return {**seed, "status": "reject", "why": "output-mismatch"}
    # idiomatic canonicalization for loop-bearing programs
    if "for " in curt_src or "while " in curt_src:
        densified = run_curt("dense", curt_src)
        if densified and densified != curt_src:
            regot = run_curt("run", densified)
            if regot is not None and norm_eq(regot, expected):
                curt_src = densified
    # fmt canonical form
    canon = run_curt("fmt", curt_src)
    if canon:
        recheck = run_curt("run", canon)
        if recheck is not None and norm_eq(recheck, expected):
            curt_src = canon
    return {
        "id": seed["id"],
        "family": seed["family"],
        "instruction": seed["instruction"],
        "curt": curt_src,
        "python": seed["python"],
        "expected": expected,
        "params": seed["params"],
        "status": "ok",
    }


def main():
    golden_only = "--golden" in sys.argv
    seeds = gen()
    if golden_only:
        # representative slice: first 3 of every family
        by = {}
        picked = []
        for s in seeds:
            if by.setdefault(s["family"], 0) < 3:
                by[s["family"]] += 1
                picked.append(s)
        seeds = picked

    with ThreadPoolExecutor(max_workers=8) as ex:
        results = list(ex.map(process, seeds))

    ok = [r for r in results if r["status"] == "ok"]
    rejects = [r for r in results if r["status"] != "ok"]
    taxonomy = {}
    for r in rejects:
        taxonomy[r["why"]] = taxonomy.get(r["why"], 0) + 1

    print(f"seeds: {len(results)}  verified: {len(ok)}  rejected: {len(rejects)} "
          f"({100 * len(rejects) / max(1, len(results)):.1f}%)")
    for why, n in sorted(taxonomy.items(), key=lambda kv: -kv[1]):
        print(f"  {why:<28} {n}")

    if not golden_only:
        OUT.mkdir(parents=True, exist_ok=True)
        with gzip.open(OUT / "pairs.jsonl.gz", "wt") as f:
            for r in ok:
                f.write(json.dumps(r, sort_keys=True) + "\n")
        with (OUT / "REPORT.md").open("w") as f:
            f.write("# py2curt pipeline report\n\n")
            f.write(f"- seeds: {len(results)}\n- verified pairs: {len(ok)}\n")
            f.write(f"- rejection rate: {100 * len(rejects) / max(1, len(results)):.1f}%\n\n")
            f.write("| rejection cause | count |\n|---|---|\n")
            for why, n in sorted(taxonomy.items(), key=lambda kv: -kv[1]):
                f.write(f"| {why} | {n} |\n")
            fams = {}
            for r in ok:
                fams[r["family"]] = fams.get(r["family"], 0) + 1
            f.write("\n| family | verified pairs |\n|---|---|\n")
            for fam, n in sorted(fams.items()):
                f.write(f"| {fam} | {n} |\n")
        print(f"wrote {OUT / 'pairs.jsonl.gz'} + REPORT.md")
    return 0 if (golden_only and not rejects) or not golden_only else 1


if __name__ == "__main__":
    sys.exit(main())
