#!/usr/bin/env python3
"""Diagnostics arms-race tournament (vz-diag, beat-zero saga 3/5).

Four renderings of THE SAME underlying curt error, fed back for repair:

  A  shipped diag verbatim: {"err","at","msg","fix"} — fix is a canned
     per-error-class teaching hint (what curt emits today)
  B  typed steelman of Zerolang's design: stable code + structured
     expected/actual + repair{id,summary} operation — NO prose hint
     (Zero v0.3.2 schema semantics at curt's single-line economy)
  C  hybrid: B + A's fix string (the ADAPT candidate)
  D  B + rustc-style machine-applicable replacement payload {line,new}
     derived by diffing broken vs the VERIFIED fixed program — an
     ORACLE-ASSISTED upper bound on compiler fix synthesis, single-turn
     only, excluded from the convergence metric
  E  the SHIPPED toolchain with REAL synthesis (fix-synthesis chunk):
     today's `curt check/run` re-derives the diag fresh, including any
     repair.replacement the compiler's generate-and-validate synthesizer
     (src/repair.rs) emitted. A/B/C protocol (<=2 turns). `payload`
     subcommand measures coverage + mechanical correctness with no API.

Corpus: data/py2curt/repairs.jsonl.gz (toolchain-verified triples).
Admission: stored diag parses; today's `curt run broken` reproduces a
parseable diag; `curt run fixed` exits 0 (its stdout is the oracle).
Stratified round-robin over distinct (err, msg-prefix) groups, seeded.

Protocol (pre-registered, think:131+132): haiku, 1 sample/cell, temp 1.0,
arms A/B/C get <=2 turns (turn 2 re-renders the new error in-arm;
wrong-output feedback shows the program's own stdout only), same
cheatsheet system prompt across arms. Transcripts frozen verbatim to
tools/bench/tourney/{arm}.jsonl; `report` re-derives deterministically.
"""

import argparse
import difflib
import gzip
import json
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from loop import (  # noqa: E402
    CURT, ENC, MODELS, PRICES, SYSTEMS, api_key, call_model, extract_code,
    norm_eq,
)

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
TOURNEY = HERE / "tourney"
REPAIRS = ROOT / "data" / "py2curt" / "repairs.jsonl.gz"
N_CORPUS = 32
SEED = 42
ARMS = ["A", "B", "C", "D", "E"]
MODEL = "haiku"

REPAIR_IDS = {
    "type_mismatch": ("align-types", "make the operand types agree"),
    "expected": ("fix-syntax", "correct the syntax at the span"),
    "unknown_name": ("define-or-rename", "define the name or fix its spelling"),
    "unknown_field": ("use-existing-field", "use a field the record declares"),
    "unexpected_char": ("remove-char", "remove the invalid character"),
}
DEFAULT_REPAIR = ("manual-review", "inspect the diagnostic and repair manually")


def run_curt(src: str, tmp: pathlib.Path, mode: str = "run") -> tuple[str, str, int]:
    tmp.write_text(src)
    p = subprocess.run([str(CURT), mode, str(tmp)], capture_output=True,
                       text=True, timeout=20)
    return p.stdout, p.stderr.strip().splitlines()[-1] if p.stderr.strip() else "", p.returncode


def today_diag(src: str, tmp: pathlib.Path) -> dict | None:
    """check-then-run pipeline; return the first failure's parsed diag."""
    for mode in ("check", "run"):
        try:
            _, err, code = run_curt(src, tmp, mode)
        except subprocess.TimeoutExpired:
            return None
        if code == 0:
            continue
        if not err:
            return None
        try:
            d = json.loads(err)
        except json.JSONDecodeError:
            return None
        return d if "err" in d else None
    return None


def parse_at(d: dict) -> tuple[int, int]:
    m = re.match(r"(\d+):(\d+)", d.get("at", "") or "")
    return (int(m.group(1)), int(m.group(2))) if m else (0, 0)


def typed_fields(d: dict) -> dict:
    """Mechanically derive Zero-style typed fields from a curt diag."""
    msg = d.get("msg", "")
    line, col = parse_at(d)
    out: dict = {"severity": "error", "code": d["err"],
                 "at": {"line": line, "col": col}}
    m = re.match(r"expected (.+?), (?:got|found) (.+)$", msg)
    if m:
        out["expected"], out["actual"] = m.group(1), m.group(2)
    m = re.match(r"`?(.+?)`? is not defined", msg)
    if m:
        out["symbol"] = m.group(1)
    m = re.match(r"(.+?) is not callable", msg)
    if m:
        out["callee"] = m.group(1)
    rid, summary = REPAIR_IDS.get(d["err"], DEFAULT_REPAIR)
    out["repair"] = {"id": rid, "summary": summary}
    return out


def replacement_payload(broken: str, fixed: str) -> tuple[list[dict], int]:
    """rustc-style machine-applicable replacements: changed lines, capped at 3."""
    bl, fl = broken.splitlines(), fixed.splitlines()
    ops = difflib.SequenceMatcher(None, bl, fl).get_opcodes()
    reps = []
    for tag, i1, i2, j1, j2 in ops:
        if tag in ("replace", "insert", "delete"):
            reps.append({"line": i1 + 1, "new": "\n".join(fl[j1:j2])})
    return reps[:3], len(reps)


def render(arm: str, d: dict, fix_hint: str, payload: list[dict] | None) -> str:
    if arm == "A":
        return json.dumps(d, separators=(",", ":"))
    t = typed_fields(d)
    if arm == "C":
        t["fix"] = fix_hint
    if arm == "D" and payload is not None:
        t["repair"]["replacement"] = payload
    return json.dumps(t, separators=(",", ":"))


def cmd_sample(_args: argparse.Namespace) -> int:
    import random
    rng = random.Random(SEED)
    tmp = pathlib.Path("/tmp/tourney_probe.curt")
    groups: dict[tuple, list] = {}
    with gzip.open(REPAIRS, "rt") as f:
        rows = [json.loads(ln) for ln in f]
    rng.shuffle(rows)
    admitted = dropped = 0
    for r in rows:
        try:
            stored = json.loads(r["diagnostic"])
        except json.JSONDecodeError:
            dropped += 1
            continue
        d = today_diag(r["broken"], tmp)
        if d is None:
            dropped += 1
            continue
        try:
            oracle, _, code = run_curt(r["fixed"], tmp)
        except subprocess.TimeoutExpired:
            dropped += 1
            continue
        if code != 0 or not oracle:
            dropped += 1
            continue
        key = (d["err"], d.get("msg", "")[:24])
        groups.setdefault(key, []).append(
            {"id": r["id"], "instruction": r["instruction"],
             "broken": r["broken"], "fixed": r["fixed"], "oracle": oracle,
             "diag": d, "stored_diag": stored})
        admitted += 1
        if admitted >= 400:  # plenty to stratify from
            break
    # round-robin across groups for diversity
    corpus, keys = [], sorted(groups, key=lambda k: (-len(groups[k]), k))
    while len(corpus) < N_CORPUS and any(groups.values()):
        for k in keys:
            if groups[k] and len(corpus) < N_CORPUS:
                corpus.append(groups[k].pop(0))
    TOURNEY.mkdir(exist_ok=True)
    with (TOURNEY / "corpus.jsonl").open("w") as f:
        for c in corpus:
            f.write(json.dumps(c) + "\n")
    dist: dict[str, int] = {}
    for c in corpus:
        dist[c["diag"]["err"]] = dist.get(c["diag"]["err"], 0) + 1
    print(f"corpus: {len(corpus)} items from {len(groups)} (err,msg) groups; "
          f"probed {admitted + dropped} rows ({dropped} dropped)")
    print("err distribution:", json.dumps(dist))
    return 0


def apply_payload(src: str, reps: list[dict]) -> str:
    """Apply {line,new} whole-line replacements (1-based; new may be multi-line)."""
    lines = src.splitlines()
    for r in sorted(reps, key=lambda r: -r["line"]):
        i = r["line"] - 1
        if 0 <= i < len(lines):
            lines[i:i + 1] = r["new"].split("\n")
    return "\n".join(lines) + "\n"


def cmd_payload(_args: argparse.Namespace) -> int:
    """Mechanical synthesis audit on the frozen corpus — no API calls.

    coverage: % of corpus errors for which today's toolchain emits a
    repair.replacement. correct: applying it yields the oracle's stdout.
    plausible-but-wrong: the payload checks clean (by construction) but the
    patched program's output differs from the oracle — APR's overfitting
    rate, the cost of a wrong payload.
    """
    corpus = [json.loads(ln) for ln in (TOURNEY / "corpus.jsonl").open()]
    tmp = pathlib.Path("/tmp/tourney_payload.curt")
    rows: list[dict] = []
    by_err: dict[str, list[str]] = {}
    for item in corpus:
        d = today_diag(item["broken"], tmp)
        reps = (d or {}).get("repair", {}).get("replacement")
        verdict = "none"
        if reps:
            patched = apply_payload(item["broken"], reps)
            try:
                stdout, _, code = run_curt(patched, tmp)
            except subprocess.TimeoutExpired:
                stdout, code = "", 1
            if code == 0 and norm_eq(stdout, item["oracle"]):
                verdict = "correct"
            else:
                verdict = "wrong"
        rows.append({"id": item["id"], "err": d["err"] if d else "?",
                     "verdict": verdict, "replacement": reps})
        by_err.setdefault(rows[-1]["err"], []).append(verdict)
        print(f"{item['id']} {rows[-1]['err']:14s} {verdict}", flush=True)
    n = len(rows)
    cov = sum(1 for r in rows if r["verdict"] != "none")
    cor = sum(1 for r in rows if r["verdict"] == "correct")
    wrong = sum(1 for r in rows if r["verdict"] == "wrong")
    with (TOURNEY / "payload_audit.jsonl").open("w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    print(f"\ncoverage {cov}/{n} ({100 * cov / n:.0f}%)  "
          f"correct {cor}/{n}  plausible-but-wrong {wrong}/{n} "
          f"(wrong-payload rate {100 * wrong / cov:.0f}% of emitted)"
          if cov else f"\ncoverage 0/{n}")
    print("by err class (correct/covered/total):")
    for e, vs in sorted(by_err.items()):
        c = sum(1 for v in vs if v == "correct")
        k = sum(1 for v in vs if v != "none")
        print(f"  {e:16s} {c}/{k}/{len(vs)}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    key = api_key()
    corpus = [json.loads(ln) for ln in (TOURNEY / "corpus.jsonl").open()]
    tmp = pathlib.Path("/tmp/tourney_run.curt")
    total_cost = 0.0
    # A-D transcripts are FROZEN (vz-diag); default reruns only the new arm
    arms = args.arms.split(",") if args.arms else ["E"]
    for arm in arms:
        out = TOURNEY / f"{arm}.jsonl"
        cells = []
        for item in corpus:
            payload = None
            if arm == "D":
                payload, n_reps = replacement_payload(item["broken"], item["fixed"])
                if n_reps > 3:
                    print(f"D {item['id']}: payload capped 3/{n_reps} hunks", flush=True)
            if arm == "E":
                # the shipped toolchain, fresh: typed fields + any synthesized
                # repair.replacement, exactly as `curt check/run` emits today
                fresh = today_diag(item["broken"], tmp)
                if fresh is None:
                    print(f"E {item['id']}: no longer errors under today's "
                          f"toolchain — feeding stored diag", flush=True)
                diag_text = (json.dumps(fresh, separators=(",", ":")) if fresh
                             else render("A", item["diag"], "", None))
            else:
                diag_text = render(arm, item["diag"], item["diag"].get("fix", ""), payload)
            max_turns = 1 if arm == "D" else 2
            turns, solved, program = [], False, item["broken"]
            feedback = diag_text
            for turn in range(max_turns):
                user = (f"Task: {item['instruction']}\n\nThis curt program is "
                        f"broken:\n```curt\n{program}```\n\nThe toolchain "
                        f"reports:\n{feedback}\n\nReply with ONLY the corrected "
                        f"curt program in a code block.")
                reply, usage, wall = call_model(MODELS[MODEL], SYSTEMS["curt"],
                                                [{"role": "user", "content": user}], key)
                turns.append({"diag_fed": feedback, "reply": reply,
                              "usage": usage, "wall_s": round(wall, 2)})
                program = extract_code(reply, "curt")
                try:
                    _, cerr, ccode = run_curt(program, tmp, "check")
                    if ccode != 0:
                        stdout, err, code = "", cerr, ccode
                    else:
                        stdout, err, code = run_curt(program, tmp)
                except subprocess.TimeoutExpired:
                    stdout, err, code = "", "timeout", 1
                if code == 0 and norm_eq(stdout, item["oracle"]):
                    solved = True
                    break
                # re-render the NEW failure in the same arm (no oracle leak)
                if code != 0 and err:
                    nd = None
                    try:
                        nd = json.loads(err)
                    except json.JSONDecodeError:
                        pass
                    if nd and "err" in nd and arm in ("B", "C"):
                        feedback = render(arm, nd, nd.get("fix", ""), None)
                    else:
                        feedback = err
                else:
                    feedback = (f"the program ran but printed:\n{stdout[:400]}\n"
                                f"which is not what the task asks for")
            cells.append({"id": item["id"], "arm": arm, "solved": solved,
                          "turns_used": len(turns),
                          "diag_tok": len(ENC.encode(diag_text)),
                          "turns": turns})
            cost = sum(
                (t["usage"].get("input_tokens", 0) * PRICES[MODELS[MODEL]][0]
                 + t["usage"].get("cache_creation_input_tokens", 0) * 1.25 * PRICES[MODELS[MODEL]][0]
                 + t["usage"].get("cache_read_input_tokens", 0) * 0.1 * PRICES[MODELS[MODEL]][0]
                 + t["usage"].get("output_tokens", 0) * PRICES[MODELS[MODEL]][1]) / 1e6
                for t in cells[-1]["turns"])
            cells[-1]["cost"] = round(cost, 6)
            total_cost += cost
            print(f"{arm} {item['id']}: {'ok' if solved else 'FAIL'} "
                  f"t={len(turns)} ${cost:.4f}", flush=True)
        with out.open("w") as f:
            for c in cells:
                f.write(json.dumps(c) + "\n")
    print(f"total spend ${total_cost:.4f}")
    return 0


def cmd_report(_args: argparse.Namespace) -> int:
    corpus = {json.loads(ln)["id"]: json.loads(ln)
              for ln in (TOURNEY / "corpus.jsonl").open()}
    print(f"{'arm':4s} {'diag-tok':>9s} {'turn1':>7s} {'final':>7s} "
          f"{'turns(solved)':>14s} {'$':>8s}")
    for arm in ARMS:
        cells = [json.loads(ln) for ln in (TOURNEY / f"{arm}.jsonl").open()]
        n = len(cells)
        dtok = sum(c["diag_tok"] for c in cells) / n
        t1 = sum(1 for c in cells if c["solved"] and c["turns_used"] == 1)
        fin = sum(1 for c in cells if c["solved"])
        conv = [c["turns_used"] for c in cells if c["solved"]]
        cost = sum(c["cost"] for c in cells)
        avg_conv = sum(conv) / len(conv) if conv else float("nan")
        print(f"{arm:4s} {dtok:9.1f} {t1:4d}/{n:<2d} {fin:4d}/{n:<2d} "
              f"{avg_conv:14.2f} {cost:8.4f}")
    # per-err breakdown, turn-1 success
    errs = sorted({c["diag"]["err"] for c in corpus.values()})
    print("\nturn-1 success by err code:")
    for e in errs:
        ids = {k for k, v in corpus.items() if v["diag"]["err"] == e}
        row = f"  {e:16s}"
        for arm in ARMS:
            cells = [json.loads(ln) for ln in (TOURNEY / f"{arm}.jsonl").open()]
            ok = sum(1 for c in cells if c["id"] in ids and c["solved"]
                     and c["turns_used"] == 1)
            row += f" {arm}:{ok}/{len(ids)}"
        print(row)
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("sample").set_defaults(fn=cmd_sample)
    runp = sub.add_parser("run")
    runp.add_argument("--arms", default="E",
                      help="comma-separated arms to (re)run; A-D are frozen")
    runp.set_defaults(fn=cmd_run)
    sub.add_parser("payload").set_defaults(fn=cmd_payload)
    sub.add_parser("report").set_defaults(fn=cmd_report)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
