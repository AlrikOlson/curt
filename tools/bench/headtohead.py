#!/usr/bin/env python3
"""vz-headtohead (beat-zero saga 4/5): curt vs Zerolang vs Python.

Pre-registered protocol (think:126, frozen before any lane): 9 shared
RosettaCode tasks (h2h/tasks.json, oracles from sha-verified programs);
each language carries its own canonical teaching doc (curt: CHEATSHEET.md;
Zero: the version-matched `zero skills get language` output, frozen in
h2h/zero-language-skill.md; Python: a one-liner); single shot + up to two
repair turns with native-format failure surfaces fed back verbatim;
2 models x 3 samples per cell; temperature 1.0; prompt caching requested
identically on every system block; transcripts frozen verbatim.

Win condition (curt vs Zero, >=2 of 3): (a) loop $/solved at success
parity +-5pp, (b) median o200k output tokens on solved cells, (c) mean
repair convergence turns on solved. The `report` subcommand computes the
verdict mechanically from the frozen lanes.

Zero execution (verified in explore-hh): fresh `zero init` project per
attempt, model .0 written to src/main.0, `zero import . --json` (its
diagnostics are the parse/type failure surface), then `zero run`.
Zero v0.3.2 sandboxed at /tmp/zero-recon per its isolation warning.
"""

import argparse
import json
import pathlib
import re
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from loop import (  # noqa: E402
    CURT, ENC, MODELS, PRICES, api_key, call_model, norm_eq,
)

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
H2H = HERE / "h2h"
ZERO_BIN = "/tmp/zero-recon/bin/zero"
SAMPLES = 3
MAX_TURNS = 3  # 1 generation + 2 repairs
LANGS = ["curt", "zero", "py"]

SYSTEMS = {
    "curt": (ROOT / "CHEATSHEET.md").read_text()
    + "\n\nYou write curt programs. Reply with ONLY a curt code block — no prose.",
    "zero": (H2H / "zero-doc.md").read_text()
    + "\n\nYou write zerolang programs (.0 projection files). Reply with ONLY a"
    " zerolang code block — no prose.",
    "py": "You write Python 3 programs. Reply with ONLY a python code block — no prose.",
}


def extract(text: str, lang: str) -> str:
    tag = {"curt": "curt", "zero": "(?:zero|zerolang|0)", "py": "(?:python|py)"}[lang]
    m = re.search(rf"```{tag}\n(.*?)```", text, re.DOTALL)
    if not m:
        m = re.search(r"```[a-z0-9]*\n?(.*?)```", text, re.DOTALL)
    return (m.group(1) if m else text).strip() + "\n"


def execute(code: str, lang: str, attempt_dir: pathlib.Path) -> tuple[str, str, int]:
    """Run code; return (stdout, failure_surface, exit_code)."""
    if lang == "py":
        p = subprocess.run([sys.executable, "-c", code], capture_output=True,
                           text=True, timeout=60)
        tail = "\n".join(p.stderr.strip().splitlines()[-12:])
        return p.stdout, tail, p.returncode
    if lang == "curt":
        f = attempt_dir / "prog.curt"
        f.write_text(code)
        chk = subprocess.run([str(CURT), "check", str(f)], capture_output=True,
                             text=True, timeout=60)
        if chk.returncode != 0:
            return "", chk.stderr.strip().splitlines()[-1], chk.returncode
        p = subprocess.run([str(CURT), "run", str(f)], capture_output=True,
                           text=True, timeout=60)
        err = p.stderr.strip().splitlines()[-1] if p.stderr.strip() else ""
        return p.stdout, err, p.returncode
    # zero: fresh project per attempt (verified flow from explore-hh)
    proj = attempt_dir / "zproj"
    if proj.exists():
        shutil.rmtree(proj)
    subprocess.run([ZERO_BIN, "init", str(proj)], capture_output=True,
                   text=True, timeout=60)
    (proj / "src").mkdir(exist_ok=True)
    (proj / "src" / "main.0").write_text(code)
    imp = subprocess.run([ZERO_BIN, "import", ".", "--json"], cwd=proj,
                         capture_output=True, text=True, timeout=60)
    try:
        ok = json.loads(imp.stdout).get("ok", False)
    except json.JSONDecodeError:
        ok = imp.returncode == 0
    if not ok:
        return "", (imp.stdout or imp.stderr).strip(), 1
    p = subprocess.run([ZERO_BIN, "run"], cwd=proj, capture_output=True,
                       text=True, timeout=120)
    err = p.stderr.strip() if p.returncode != 0 else ""
    return p.stdout, err, p.returncode


def run_cell(model: str, lang: str, task: dict, sample: int, key: str,
             attempt_dir: pathlib.Path) -> dict:
    user0 = (f"Write a {'Python 3' if lang == 'py' else {'curt': 'curt', 'zero': 'zerolang'}[lang]} "
             f"program for this task:\n\n{task['prompt']}\n\n"
             f"Reply with ONLY the program in a code block.")
    messages = [{"role": "user", "content": user0}]
    turns, solved, out_tok_solved = [], False, None
    for turn in range(MAX_TURNS):
        reply, usage, wall = call_model(MODELS[model], SYSTEMS[lang], messages, key)
        code = extract(reply, lang)
        try:
            stdout, surface, rc = execute(code, lang, attempt_dir)
        except subprocess.TimeoutExpired:
            stdout, surface, rc = "", "timeout", 1
        turns.append({"reply": reply, "usage": usage, "wall_s": round(wall, 2),
                      "exit": rc})
        if rc == 0 and norm_eq(stdout, task["oracle"]):
            solved = True
            out_tok_solved = len(ENC.encode(code))
            break
        if rc != 0 and surface:
            fb = f"The toolchain reports:\n{surface[:2000]}"
        else:
            fb = (f"The program ran but printed:\n{stdout[:400]}\n"
                  f"which is not what the task asks for.")
        messages.append({"role": "assistant", "content": reply})
        messages.append({"role": "user", "content":
                         fb + "\n\nReply with ONLY the corrected program in a code block."})
    cost = sum(
        (t["usage"].get("input_tokens", 0) * PRICES[MODELS[model]][0]
         + t["usage"].get("cache_creation_input_tokens", 0) * 1.25 * PRICES[MODELS[model]][0]
         + t["usage"].get("cache_read_input_tokens", 0) * 0.1 * PRICES[MODELS[model]][0]
         + t["usage"].get("output_tokens", 0) * PRICES[MODELS[model]][1]) / 1e6
        for t in turns)
    return {"task": task["id"], "model": model, "lang": lang, "sample": sample,
            "solved": solved, "turns_used": len(turns),
            "out_tok": out_tok_solved, "cost": round(cost, 6), "turns": turns}


def cmd_run(_args: argparse.Namespace) -> int:
    key = api_key()
    tasks = json.loads((H2H / "tasks.json").read_text())
    attempt_dir = pathlib.Path("/tmp/h2h_exec")
    attempt_dir.mkdir(exist_ok=True)
    total = 0.0
    for model in MODELS:
        for lang in LANGS:
            out = H2H / f"{model}_{lang}.jsonl"
            with out.open("w") as f:
                for task in tasks:
                    for s in range(SAMPLES):
                        cell = run_cell(model, lang, task, s, key, attempt_dir)
                        f.write(json.dumps(cell) + "\n")
                        f.flush()
                        total += cell["cost"]
                        print(f"{model} {lang} {task['id']}#{s}: "
                              f"{'ok' if cell['solved'] else 'FAIL'} "
                              f"t={cell['turns_used']} ${cell['cost']:.4f}", flush=True)
    print(f"total spend ${total:.4f}")
    return 0


def lane(model: str, lang: str) -> list[dict]:
    return [json.loads(ln) for ln in (H2H / f"{model}_{lang}.jsonl").open()]


def axes(model: str, lang: str) -> dict:
    cells = lane(model, lang)
    n = len(cells)
    solved = [c for c in cells if c["solved"]]
    cost = sum(c["cost"] for c in cells)
    outs = sorted(c["out_tok"] for c in solved)
    med_out = outs[len(outs) // 2] if outs else None
    conv = sum(c["turns_used"] for c in solved) / len(solved) if solved else None
    return {"n": n, "solved": len(solved), "rate": len(solved) / n,
            "cost": cost, "cps": cost / len(solved) if solved else None,
            "med_out": med_out, "conv": conv}


def cmd_report(_args: argparse.Namespace) -> int:
    print(f"{'model':7s}{'lang':6s}{'solved':>8s}{'$total':>9s}{'$/solved':>10s}"
          f"{'med-out-tok':>12s}{'turns':>7s}")
    stats = {}
    for model in MODELS:
        for lang_ in LANGS:
            a = axes(model, lang_)
            stats[(model, lang_)] = a
            print(f"{model:7s}{lang_:6s}{a['solved']:>5d}/{a['n']:<3d}"
                  f"{a['cost']:>8.4f} {a['cps'] if a['cps'] is not None else float('nan'):>9.4f}"
                  f"{a['med_out'] if a['med_out'] is not None else -1:>12d}"
                  f"{a['conv'] if a['conv'] is not None else float('nan'):>7.2f}")
    # mechanical verdict per pre-registration (curt vs zero, pooled across models)
    print("\npre-registered verdict (curt vs zero, per model):")
    for model in MODELS:
        c, z = stats[(model, "curt")], stats[(model, "zero")]
        parity = abs(c["rate"] - z["rate"]) <= 0.05
        a_ok = (c["cps"] is not None and z["cps"] is not None
                and c["cps"] < z["cps"] and parity)
        b_ok = (c["med_out"] is not None and z["med_out"] is not None
                and c["med_out"] < z["med_out"])
        c_ok = (c["conv"] is not None and z["conv"] is not None
                and c["conv"] < z["conv"])
        wins = sum([a_ok, b_ok, c_ok])
        print(f"  {model}: (a) $/solved@parity={'WIN' if a_ok else 'no'}"
              f" (parity={'yes' if parity else 'NO'}, {c['rate']:.2f} vs {z['rate']:.2f})"
              f"  (b) med-out={'WIN' if b_ok else 'no'} ({c['med_out']} vs {z['med_out']})"
              f"  (c) turns={'WIN' if c_ok else 'no'}"
              f" ({c['conv'] if c['conv'] else float('nan'):.2f} vs"
              f" {z['conv'] if z['conv'] else float('nan'):.2f})"
              f"  -> {wins}/3 {'CURT WINS' if wins >= 2 else 'curt does not win'}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("run").set_defaults(fn=cmd_run)
    sub.add_parser("report").set_defaults(fn=cmd_report)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
