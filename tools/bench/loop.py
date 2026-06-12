#!/usr/bin/env python3
"""agent-loop-bench: measure LOOP DOLLARS, not program tokens.

Runs full agent task loops — generate -> execute -> feed back the
language's NATIVE failure surface -> repair (<=2 repair turns) — for
curt vs Python over both frozen suites (bench 15 algorithmic, dbench 10
ceremony), recording every input/output token from the API's own usage
fields and pricing at live rates.

  loop.py run [--models haiku,sonnet] [--limit N]   # live calls; freezes JSONL
  loop.py report                                    # deterministic re-derivation

Frozen-lane discipline: transcripts under loops/ are committed verbatim,
failures included; `report` only reads frozen files, so the published
matrix reproduces from the repository alone.

Protocol (matches gen_lanes.py where applicable): each language gets its
own canonical teaching context (curt: CHEATSHEET.md; Python: a one-line
instruction — the asymmetric documentation tax is part of what is being
measured), temperature 1.0, max_tokens 900 per turn, prompt-cached
system block, conversation accumulates across repair turns (the input-
side compounding the thesis is about). Wrong-output feedback contains
the program's own stdout only — never the expected output beyond what
the task prompt already states.

Pricing (claude-api reference, cached 2026-06-04): haiku 4.5 $1/$5 per
MTok in/out, sonnet 4.6 $3/$15; cache write 1.25x input rate, cache
read 0.1x.
"""

import argparse
import json
import pathlib
import re
import subprocess
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor

import tiktoken

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CURT = ROOT / "target" / "release" / "curt"
LOOPS = HERE / "loops"
SHEET = (ROOT / "CHEATSHEET.md").read_text()

MODELS = {"haiku": "claude-haiku-4-5", "sonnet": "claude-sonnet-4-6"}
PRICES = {  # USD per MTok (input, output); write=1.25x in, read=0.1x in
    "claude-haiku-4-5": (1.00, 5.00),
    "claude-sonnet-4-6": (3.00, 15.00),
}
MAX_REPAIRS = 2
ENC = tiktoken.get_encoding("o200k_base")

SYSTEMS = {
    "curt": SHEET + "\n\nYou write curt programs. Reply with ONLY a curt code block — no prose.",
    "py": "You write Python 3 programs. Reply with ONLY a python code block — no prose.",
}


def api_key() -> str:
    import os
    return os.environ.get("ANTHROPIC_API_KEY") or (
        ROOT / "data" / "external" / "anthropic.key").read_text().strip()


def parse_prompts(path: pathlib.Path, lang: str | None) -> dict[str, str]:
    out = {}
    for m in re.finditer(r"^## (\S+)\n(.*?)(?=^## |\Z)", path.read_text(), re.M | re.S):
        prompt = m.group(2).strip()
        if lang is not None:
            prompt = prompt.replace("<LANG>", lang)
        out[m.group(1)] = prompt
    return out


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


def call_model(model: str, system: str, messages: list[dict], key: str) -> tuple[str, dict, float]:
    body = {
        "model": model, "max_tokens": 900, "temperature": 1.0,
        "system": [{"type": "text", "text": system,
                    "cache_control": {"type": "ephemeral"}}],
        "messages": messages,
    }
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(body).encode(),
        headers={"x-api-key": key, "anthropic-version": "2023-06-01",
                 "content-type": "application/json"},
    )
    for attempt in range(5):
        t0 = time.monotonic()
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                data = json.load(r)
            wall = time.monotonic() - t0
            text = "".join(b.get("text", "") for b in data["content"])
            return text, data["usage"], wall
        except Exception:  # noqa: BLE001 — retry transient API errors
            time.sleep(2 ** attempt + 1)
    raise RuntimeError(f"model call failed after retries: {model}")


def extract_code(text: str, lang: str) -> str:
    tag = "curt" if lang == "curt" else "(?:python|py)"
    m = re.search(rf"```{tag}\n(.*?)```", text, re.DOTALL)
    if not m:
        m = re.search(r"```\n?(.*?)```", text, re.DOTALL)
    return (m.group(1) if m else text).strip() + "\n"


def execute(code: str, lang: str, suite: str, workdir: pathlib.Path) -> tuple[str, str, int | None]:
    """Returns (stdout, failure_surface, returncode); returncode None = timeout."""
    src = workdir / f"prog.{ 'curt' if lang == 'curt' else 'py' }"
    src.write_text(code)
    if lang == "curt":
        cmd = [str(CURT), "run"] + (["--fs"] if suite == "dbench" else []) + [str(src)]
    else:
        cmd = ["python3", str(src)]
    cwd = (HERE.parent / "dbench" / "fixtures") if suite == "dbench" else workdir
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=30, cwd=cwd)
    except subprocess.TimeoutExpired:
        return "", "the program timed out after 30 seconds", None
    if p.returncode != 0:
        tail = "\n".join((p.stderr or p.stdout).strip().split("\n")[-12:])
        return p.stdout, tail, p.returncode
    return p.stdout, "", 0


def run_cell(model: str, lang: str, suite: str, task: str, prompt: str,
             expected: str, key: str, workdir: pathlib.Path) -> dict:
    system = SYSTEMS[lang]
    messages = [{"role": "user", "content": prompt}]
    turns, diag_feed = [], []
    solved = False
    for turn in range(1 + MAX_REPAIRS):
        text, usage, wall = call_model(model, system, messages, key)
        turns.append({"wall_s": round(wall, 2), "usage": usage, "reply": text})
        code = extract_code(text, lang)
        stdout, failure, rc = execute(code, lang, suite, workdir)
        if rc == 0 and norm_eq(stdout, expected):
            solved = True
            break
        if rc == 0:
            got = "\n".join(stdout.rstrip("\n").split("\n")[:10])[:500]
            diag = (f"The program ran but printed the wrong output. "
                    f"It printed:\n{got}\nReply with the corrected full program — ONLY a code block.")
        else:
            diag = (f"The program failed:\n{failure}\n"
                    f"Reply with the corrected full program — ONLY a code block.")
        if turn < MAX_REPAIRS:
            diag_feed.append(diag)
            messages.append({"role": "assistant", "content": text})
            messages.append({"role": "user", "content": diag})

    pin, pout = PRICES[model]
    cost = 0.0
    for t in turns:
        u = t["usage"]
        cost += (u.get("input_tokens", 0) * pin
                 + u.get("cache_creation_input_tokens", 0) * 1.25 * pin
                 + u.get("cache_read_input_tokens", 0) * 0.1 * pin
                 + u.get("output_tokens", 0) * pout) / 1e6
    return {
        "suite": suite, "task": task, "lang": lang, "model": model,
        "solved": solved, "turns_used": len(turns),
        "diag_feed_o200k": sum(len(ENC.encode(d)) for d in diag_feed),
        "diag_feed_bytes": sum(len(d.encode()) for d in diag_feed),
        "cost_usd": round(cost, 6),
        "turns": turns,
    }


def suite_specs() -> list[tuple[str, dict, pathlib.Path]]:
    return [
        ("bench", None, HERE),
        ("dbench", None, HERE.parent / "dbench"),
    ]


def load_tasks(lang: str) -> list[tuple[str, str, str, str]]:
    """(suite, task, prompt, expected)"""
    sub = {"curt": "curt", "py": "Python"}[lang]
    out = []
    for suite, _, sdir in suite_specs():
        prompts = parse_prompts(sdir / "PROMPTS.md", sub if suite == "bench" else None)
        for task, prompt in sorted(prompts.items()):
            exp = sdir / "tasks" / f"{task}.expected"
            if exp.exists():
                out.append((suite, task, prompt, exp.read_text()))
    return out


def cmd_run(args: argparse.Namespace) -> int:
    import tempfile
    key = api_key()
    LOOPS.mkdir(exist_ok=True)
    shorts = args.models.split(",")
    jobs = []
    for short in shorts:
        model = MODELS[short]
        for lang in ("curt", "py"):
            tasks = load_tasks(lang)
            if args.limit:
                tasks = tasks[: args.limit]
            for suite, task, prompt, expected in tasks:
                jobs.append((short, model, lang, suite, task, prompt, expected))
    print(f"{len(jobs)} loop cells")

    def one(job):
        short, model, lang, suite, task, prompt, expected = job
        with tempfile.TemporaryDirectory() as td:
            return short, run_cell(model, lang, suite, task, prompt, expected,
                                   key, pathlib.Path(td))

    results: dict[str, list[dict]] = {}
    with ThreadPoolExecutor(max_workers=8) as ex:
        for short, row in ex.map(one, jobs):
            results.setdefault(f"{short}_{row['lang']}", []).append(row)
            mark = "ok " if row["solved"] else "FAIL"
            print(f"  {mark} {short:6s} {row['lang']:4s} {row['suite']}/{row['task']}"
                  f"  turns={row['turns_used']} ${row['cost_usd']:.4f}")

    for name, rows in sorted(results.items()):
        dest = LOOPS / f"{name}.jsonl"
        rows.sort(key=lambda r: (r["suite"], r["task"]))
        dest.write_text("".join(json.dumps(r) + "\n" for r in rows))
        print(f"froze {dest} ({len(rows)} cells)")
    return 0


def cmd_report(_args: argparse.Namespace) -> int:
    files = sorted(LOOPS.glob("*.jsonl"))
    if not files:
        print("no frozen loops; run `loop.py run` first")
        return 1
    rows = [json.loads(line) for f in files for line in f.read_text().splitlines()]
    total_spend = sum(r["cost_usd"] for r in rows)

    print("=== loop dollars (full task loop: generate + <=2 native-diagnostic repairs) ===")
    print(f"{'model':8s} {'lang':5s} {'solved':>9s} {'$total':>8s} {'$/solved':>9s} "
          f"{'turns':>6s} {'in-tok':>9s} {'out-tok':>8s} {'diag-tok':>8s}")
    for model in PRICES:
        for lang in ("curt", "py"):
            sel = [r for r in rows if r["model"] == model and r["lang"] == lang]
            if not sel:
                continue
            solved = [r for r in sel if r["solved"]]
            cost = sum(r["cost_usd"] for r in sel)
            per = cost / len(solved) if solved else float("inf")
            turns = sum(r["turns_used"] for r in solved) / len(solved) if solved else 0
            tin = sum(u["usage"].get("input_tokens", 0)
                      + u["usage"].get("cache_creation_input_tokens", 0)
                      + u["usage"].get("cache_read_input_tokens", 0)
                      for r in sel for u in r["turns"])
            tout = sum(u["usage"].get("output_tokens", 0) for r in sel for u in r["turns"])
            dtok = sum(r["diag_feed_o200k"] for r in sel)
            short = [k for k, v in MODELS.items() if v == model][0]
            print(f"{short:8s} {lang:5s} {len(solved):3d}/{len(sel):3d}   "
                  f"${cost:7.4f} ${per:8.4f} {turns:6.2f} {tin:9d} {tout:8d} {dtok:8d}")

    print("\n=== diagnostic-size contribution (repair-turn feedback only) ===")
    for lang in ("curt", "py"):
        sel = [r for r in rows if r["lang"] == lang and r["diag_feed_o200k"] > 0]
        if sel:
            tok = sum(r["diag_feed_o200k"] for r in sel)
            byt = sum(r["diag_feed_bytes"] for r in sel)
            print(f"{lang:5s} {len(sel):3d} cells with repairs, "
                  f"{tok:6d} o200k tokens fed back ({byt} bytes, "
                  f"{tok / len(sel):6.1f} tok/cell)")
        else:
            print(f"{lang:5s} no repair turns")

    print(f"\ntotal live spend across all frozen loops: ${total_spend:.4f}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    runp = sub.add_parser("run")
    runp.add_argument("--models", default="haiku,sonnet")
    runp.add_argument("--limit", type=int, default=0)
    runp.set_defaults(fn=cmd_run)
    rep = sub.add_parser("report")
    rep.set_defaults(fn=cmd_report)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
