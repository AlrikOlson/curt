#!/usr/bin/env python3
"""Sheet-cache experiment (chunk:sheet-cache, think:154).

Reruns the haiku curt loop lane (identical protocol to loop.py) under
two sheet variants — ext.md (>4096 Anthropic tokens of execution-
verified worked examples, so haiku's cache floor is cleared) and
lite.md (≤800 o200k distillation) — freezing to
loops/haiku_curt_{ext,lite}.jsonl. The frozen baseline
(loops/haiku_curt.jsonl) is never touched. `report` prints the
three-way comparison with cache-field evidence.
"""

import argparse
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import loop  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
SHEETS = HERE / "sheets"
LOOPS = HERE / "loops"
SUFFIX = "\n\nYou write curt programs. Reply with ONLY a curt code block — no prose."


def cmd_run(args: argparse.Namespace) -> int:
    key = loop.api_key()
    variant = args.variant
    loop.SYSTEMS["curt"] = (SHEETS / f"{variant}.md").read_text() + SUFFIX
    tasks = loop.load_tasks("curt")
    out = LOOPS / f"haiku_curt_{variant}.jsonl"
    total = 0.0
    with tempfile.TemporaryDirectory() as td, out.open("w") as f:
        wd = pathlib.Path(td)
        for suite, task, prompt, expected in tasks:
            cell = loop.run_cell(loop.MODELS["haiku"], "curt", suite, task,
                                 prompt, expected, key, wd)
            f.write(json.dumps(cell) + "\n")
            f.flush()
            total += cell["cost_usd"]
            print(f"{variant} {suite}/{task}: {'ok' if cell['solved'] else 'FAIL'} "
                  f"t={cell['turns_used']} ${cell['cost_usd']:.4f}", flush=True)
    print(f"{variant} total ${total:.4f}")
    return 0


def lane_stats(path: pathlib.Path) -> dict:
    cells = [json.loads(ln) for ln in path.open()]
    solved = sum(c["solved"] for c in cells)
    cost = sum(c["cost_usd"] for c in cells)
    usage = [t["usage"] for c in cells for t in c["turns"]]
    return {
        "n": len(cells), "solved": solved, "cost": cost,
        "cps": cost / solved if solved else float("nan"),
        "in": sum(u.get("input_tokens", 0) for u in usage),
        "cc": sum(u.get("cache_creation_input_tokens", 0) for u in usage),
        "cr": sum(u.get("cache_read_input_tokens", 0) for u in usage),
    }


def cmd_report(_args: argparse.Namespace) -> int:
    rows = {
        "baseline (2.0k sheet)": LOOPS / "haiku_curt.jsonl",
        "ext (4.6k sheet)": LOOPS / "haiku_curt_ext.jsonl",
        "lite (0.4k sheet)": LOOPS / "haiku_curt_lite.jsonl",
    }
    print(f"{'variant':24s}{'solved':>8s}{'$total':>9s}{'$/solved':>10s}"
          f"{'in':>9s}{'cache-wr':>10s}{'cache-rd':>10s}")
    for name, path in rows.items():
        if not path.exists():
            print(f"{name:24s}  (missing)")
            continue
        s = lane_stats(path)
        print(f"{name:24s}{s['solved']:>5d}/{s['n']:<3d}{s['cost']:>8.4f} "
              f"{s['cps']:>9.4f}{s['in']:>9d}{s['cc']:>10d}{s['cr']:>10d}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("variant", choices=["ext", "lite"])
    r.set_defaults(fn=cmd_run)
    sub.add_parser("report").set_defaults(fn=cmd_report)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
