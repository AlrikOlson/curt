#!/usr/bin/env python3
"""Dense-read parity test (hypothesis hx3, think:145).

CLAIM: an agent re-reading curt (denser, with its sheet) loses no
comprehension accuracy vs re-reading Python — the density discount
applies to generation only. This tests the PRACTICAL COMPOUND claim
(density + unfamiliarity + doc help together); a curt deficit cannot
be attributed to density alone, stated in advance.

FROZEN DESIGN (think:145, before any API call): n=40 pairs from
data/py2curt/pairs.jsonl.gz (same algorithm in both languages,
execution-verified, shared expected stdout) — seed 42, curt 8-30
lines, stdout 1-10 lines, both members re-verified clean. Probes:
  P1 output prediction — show the program, ask for exact stdout
     (graded norm_eq vs expected).
  P2 bug localization — one seeded mechanical mutation (operator flip
     or constant bump) per member; show mutated program + intended
     stdout, ask for the edited line number (graded exact).
2 languages x 2 probes x 2 models x 40 pairs = 320 calls, temp 1.0,
1 sample. Own-docs convention: curt probes carry CHEATSHEET.md,
python an 18-token line.

FROZEN PREDICTION: accuracy(curt) >= accuracy(python) - 5pp on EACH
probe family, per model. REFUTATION: curt deficit > 5pp on either
family for either model.
"""

import argparse
import gzip
import json
import pathlib
import random
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from loop import CURT, ENC, MODELS, PRICES, api_key, call_model, norm_eq  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
DR = HERE / "densread"
PAIRS = ROOT / "data" / "py2curt" / "pairs.jsonl.gz"
N = 40
SEED = 42

SYSTEMS = {
    "curt": (ROOT / "CHEATSHEET.md").read_text()
    + "\n\nYou read curt programs and answer questions about them precisely.",
    "py": "You read Python 3 programs and answer questions about them precisely.",
}

MUT_OPS = [("+", "-"), ("-", "+"), ("*", "+"), ("<", "<="), (">", ">=")]


def run_prog(src: str, lang: str, tmp: pathlib.Path) -> tuple[str, int]:
    if lang == "py":
        p = subprocess.run([sys.executable, "-c", src], capture_output=True,
                           text=True, timeout=20)
    else:
        tmp.write_text(src)
        p = subprocess.run([str(CURT), "run", str(tmp)], capture_output=True,
                           text=True, timeout=20)
    return p.stdout, p.returncode


def mutate(src: str, rng: random.Random) -> tuple[str, int] | None:
    """One mechanical mutation on a random eligible line. Returns (src, line_no 1-based)."""
    lines = src.splitlines()
    cands = []
    for i, ln in enumerate(lines):
        # skip strings-heavy lines to keep mutations semantic, not textual
        stripped = re.sub(r'"[^"]*"', "", ln)
        for a, _b in MUT_OPS:
            if f" {a} " in stripped:
                cands.append((i, "op", a))
                break
        else:
            if re.search(r"(?<![\w.])\d+(?![\w.])", stripped):
                cands.append((i, "num", None))
    if not cands:
        return None
    i, kind, op = rng.choice(cands)
    if kind == "op":
        b = dict(MUT_OPS)[op]
        lines[i] = lines[i].replace(f" {op} ", f" {b} ", 1)
    else:
        m = re.search(r"(?<![\w.])(\d+)(?![\w.])", re.sub(r'"[^"]*"', lambda x: "_" * len(x.group()), lines[i]))
        if not m:
            return None
        s, e = m.span(1)
        lines[i] = lines[i][:s] + str(int(m.group(1)) + 1) + lines[i][e:]
    return "\n".join(lines) + "\n", i + 1


def cmd_sample(_args: argparse.Namespace) -> int:
    rng = random.Random(SEED)
    tmp = pathlib.Path("/tmp/dr_probe.curt")
    rows = [json.loads(ln) for ln in gzip.open(PAIRS, "rt")]
    rng.shuffle(rows)
    out, dropped = [], 0
    for r in rows:
        if len(out) >= N:
            break
        curt_src, py_src, exp = r["curt"], r["python"], r["expected"]
        nl = curt_src.count("\n")
        if not (8 <= nl <= 30 and 1 <= exp.count("\n") + bool(exp and not exp.endswith("\n")) <= 10):
            continue
        try:
            co, cc = run_prog(curt_src, "curt", tmp)
            po, pc = run_prog(py_src, "py", tmp)
        except subprocess.TimeoutExpired:
            dropped += 1
            continue
        if cc != 0 or pc != 0 or not norm_eq(co, exp) or not norm_eq(po, exp):
            dropped += 1
            continue
        muts = {}
        ok = True
        for lang, src in (("curt", curt_src), ("py", py_src)):
            m = mutate(src, rng)
            if m is None:
                ok = False
                break
            # mutated program must NOT still produce the expected output
            try:
                mo, mc = run_prog(m[0], lang, tmp)
            except subprocess.TimeoutExpired:
                mo, mc = "", 1
            if mc == 0 and norm_eq(mo, exp):
                ok = False  # silent mutation — resample pair
                break
            muts[lang] = {"src": m[0], "line": m[1]}
        if not ok:
            dropped += 1
            continue
        out.append({"id": r["id"], "expected": exp, "curt": curt_src,
                    "py": py_src, "mut": muts,
                    "tok": {"curt": len(ENC.encode(curt_src)),
                            "py": len(ENC.encode(py_src))}})
    DR.mkdir(exist_ok=True)
    with (DR / "probes.jsonl").open("w") as f:
        for o in out:
            f.write(json.dumps(o) + "\n")
    ctok = sum(o["tok"]["curt"] for o in out) / len(out)
    ptok = sum(o["tok"]["py"] for o in out) / len(out)
    print(f"probes: {len(out)} pairs ({dropped} dropped); "
          f"mean tokens curt {ctok:.0f} vs py {ptok:.0f} ({ptok/ctok:.2f}x)")
    return 0


def cmd_run(_args: argparse.Namespace) -> int:
    key = api_key()
    probes = [json.loads(ln) for ln in (DR / "probes.jsonl").open()]
    total = 0.0
    for model in MODELS:
        cells = []
        for p in probes:
            for lang in ("curt", "py"):
                lname = "curt" if lang == "curt" else "Python 3"
                u1 = (f"What exactly does this {lname} program print? "
                      f"Reply with ONLY the stdout, nothing else.\n\n```\n{p[lang]}```")
                u2 = (f"This {lname} program was correct and printed exactly:\n"
                      f"```\n{p['expected']}```\nExactly one line was then edited, "
                      f"breaking it:\n```\n{p['mut'][lang]['src']}```\n"
                      f"Which line number was edited? Reply with ONLY the number.")
                for probe, user in (("output", u1), ("bugloc", u2)):
                    reply, usage, _w = call_model(MODELS[model], SYSTEMS[lang],
                                                  [{"role": "user", "content": user}], key)
                    if probe == "output":
                        ok = norm_eq(reply.strip().strip("`").strip(), p["expected"])
                    else:
                        m = re.search(r"\d+", reply)
                        ok = bool(m) and int(m.group()) == p["mut"][lang]["line"]
                    cost = (usage.get("input_tokens", 0) * PRICES[MODELS[model]][0]
                            + usage.get("cache_creation_input_tokens", 0) * 1.25 * PRICES[MODELS[model]][0]
                            + usage.get("cache_read_input_tokens", 0) * 0.1 * PRICES[MODELS[model]][0]
                            + usage.get("output_tokens", 0) * PRICES[MODELS[model]][1]) / 1e6
                    total += cost
                    cells.append({"id": p["id"], "lang": lang, "probe": probe,
                                  "ok": ok, "reply": reply, "usage": usage,
                                  "cost": round(cost, 6)})
                    print(f"{model} {lang} {probe} {p['id']}: {'ok' if ok else 'X'}",
                          flush=True)
        with (DR / f"{model}.jsonl").open("w") as f:
            for c in cells:
                f.write(json.dumps(c) + "\n")
    print(f"total spend ${total:.4f}")
    return 0


def grade(cell: dict, probes: dict) -> bool:
    """Re-grade from the frozen reply. Bug-localization extraction fixed
    after audit: models reason in prose despite the only-the-number
    instruction, and a first-number regex grabs output values; prefer the
    LAST 'line N' mention, else the LAST standalone integer. The frozen
    prediction is unchanged — this corrects the measuring instrument."""
    p = probes[cell["id"]]
    if cell["probe"] == "output":
        return norm_eq(cell["reply"].strip().strip("`").strip(), p["expected"])
    want = p["mut"][cell["lang"]]["line"]
    ms = re.findall(r"[Ll]ine\s*#?\s*(\d+)", cell["reply"])
    if ms:
        return int(ms[-1]) == want
    ms = re.findall(r"\d+", cell["reply"])
    return bool(ms) and int(ms[-1]) == want


def cmd_report(_args: argparse.Namespace) -> int:
    probes = {json.loads(ln)["id"]: json.loads(ln)
              for ln in (DR / "probes.jsonl").open()}
    print(f"{'model':8s}{'probe':8s}{'curt':>8s}{'py':>8s}{'delta':>8s}")
    worst = 0.0
    for model in MODELS:
        cells = [json.loads(ln) for ln in (DR / f"{model}.jsonl").open()]
        for probe in ("output", "bugloc"):
            acc = {}
            for lang in ("curt", "py"):
                sel = [c for c in cells if c["lang"] == lang and c["probe"] == probe]
                acc[lang] = sum(grade(c, probes) for c in sel) / len(sel)
            d = acc["curt"] - acc["py"]
            worst = min(worst, d)
            print(f"{model:8s}{probe:8s}{acc['curt']:>8.2f}{acc['py']:>8.2f}{d:>+8.2f}")
    verdict = "HOLDS" if worst >= -0.05 else "REFUTED"
    print(f"\nfrozen prediction (curt >= py - 5pp on every model x probe): "
          f"{verdict} (worst delta {worst:+.2f})")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("sample").set_defaults(fn=cmd_sample)
    sub.add_parser("run").set_defaults(fn=cmd_run)
    sub.add_parser("report").set_defaults(fn=cmd_report)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
