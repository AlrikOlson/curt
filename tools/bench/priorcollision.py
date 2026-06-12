#!/usr/bin/env python3
"""Prior-collision / uncanny-valley-syntax test (hypothesis hx1, think:143).

FROZEN RUBRIC (locked in the reasoning trace before any counting):
construct families pre-assigned to a similarity class relative to Python
(the dominant prior). Exposure unit = source LINES containing a family's
signature, summed over all first-turn generated curt programs in the
frozen lanes. Attribution: for first-turn toolchain failures, the
families present on the diagnostic's reported line each receive one
attributed failure (multi-family lines counted in every family present —
published as a known approximation). Wrong-output runs count as
exposures only. Per-class rate = attributed failures / exposed lines.

FROZEN PREDICTION (non-monotone): rate(similar-divergent) > rate(alien)
AND rate(similar-divergent) > rate(identical).
REFUTATION: rate(alien) >= rate(similar-divergent), or max/min < 2x.

Lanes: bench+dbench answers v4/v5 (haiku+sonnet, s1-s3) and h2h curt
lanes (both models, first turns). Loop lanes excluded (same task suites
as v4/v5; noted). Failures are re-derived under the CURRENT toolchain
(consistent across all programs; may differ from generation-time grades).
"""

import json
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CURT = ROOT / "target" / "release" / "curt"
DBENCH = ROOT / "tools" / "dbench"

FAMILIES = {  # name: (class, line regex)
    "if-else": ("identical", r"\bif\b|\belse\b"),
    "while": ("identical", r"\bwhile\b"),
    "for-in": ("identical", r"\bfor\b.*\bin\b"),
    "list-lit": ("identical", r"\["),
    "string-lit": ("identical", r'"'),
    "range": ("similar", r"\brange\b"),
    "bare-call-print": ("similar", r"\bprint\b"),
    "conv-method": ("similar", r"\.(int|float|str)\b"),
    "interp-brace": ("similar", r'"[^"]*\{'),
    "lambda-arrow": ("similar", r"->"),
    "equation-def": ("similar", r"^[a-z_]\w*(\s+\w+)+\s*="),
    "slice": ("similar", r"\[\s*[^\]]*:"),
    "pipeline-union": ("alien", r"\|"),
    "rescue-q": ("alien", r"\?"),
    "match-err": ("alien", r"\bmatch\b|\berr\b"),
    "go": ("alien", r"\bgo\b"),
    "capability": ("alien", r"\bfs\.|\bnet\."),
    "record-lit": ("alien", r"[A-Z]\w*\{"),
    "sig": ("alien", r"::"),
    "stdlib-verb": ("alien", r"\.(sum|fold|join|lines|chars|sort|rev|upper|lower|keys|vals|len|map|filter|split|trim|words|json)\b"),
}
# '->' inside a match block belongs to match-err (alien), not lambda.


def families_on(line: str, in_match: bool) -> set[str]:
    out = set()
    for name, (_, rx) in FAMILIES.items():
        if re.search(rx, line):
            out.add(name)
    if "->" in line and in_match:
        out.discard("lambda-arrow")
        out.add("match-err")
    return out


def match_depth_lines(src: str) -> list[bool]:
    """Per line: are we inside a match block?"""
    flags, depth = [], 0
    for line in src.splitlines():
        entering = re.search(r"\bmatch\b", line) is not None
        flags.append(depth > 0 or entering)
        if entering:
            depth += line.count("{") - line.count("}") or 1
        elif depth > 0:
            depth += line.count("{") - line.count("}")
            depth = max(depth, 0)
    return flags


def run_program(src: str, kind: str, tmp: pathlib.Path) -> tuple[str, dict | None, int]:
    """Returns (stdout, diag-or-None, exit). kind: bench|dbench|h2h."""
    tmp.write_text(src)
    chk = subprocess.run([str(CURT), "check", str(tmp)], capture_output=True,
                         text=True, timeout=30)
    if chk.returncode != 0:
        return "", parse_diag(chk.stderr), chk.returncode
    cmd = [str(CURT), "run"] + (["--fs"] if kind == "dbench" else []) + [str(tmp)]
    cwd = DBENCH / "fixtures" if kind == "dbench" else None
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=30, cwd=cwd)
    except subprocess.TimeoutExpired:
        return "", None, 1
    return p.stdout, parse_diag(p.stderr) if p.returncode != 0 else None, p.returncode


def parse_diag(stderr: str) -> dict | None:
    line = stderr.strip().splitlines()[-1] if stderr.strip() else ""
    try:
        d = json.loads(line)
        return d if "err" in d else None
    except json.JSONDecodeError:
        return None


def collect() -> list[tuple[str, str, str]]:
    """(source, kind, lane) for all first-turn curt programs."""
    progs = []
    for model in ("haiku", "sonnet"):
        for v in ("v4", "v5"):
            for suite, kind in ((HERE, "bench"), (DBENCH, "dbench")):
                base = suite / "answers" / f"curt_{model}_{v}"
                if not base.exists():
                    continue
                for s in sorted(base.glob("s*/*.curt")):
                    progs.append((s.read_text(), kind, f"{kind}-{model}-{v}"))
        lane = HERE / "h2h" / f"{model}_curt.jsonl"
        for ln in lane.open():
            c = json.loads(ln)
            reply = c["turns"][0]["reply"]
            m = re.search(r"```[a-z0-9]*\n?(.*?)```", reply, re.DOTALL)
            progs.append(((m.group(1) if m else reply).strip() + "\n", "h2h",
                          f"h2h-{model}"))
    return progs


def main() -> int:
    progs = collect()
    tmp = pathlib.Path("/tmp/pc_probe.curt")
    exposed: dict[str, int] = {f: 0 for f in FAMILIES}
    attributed: dict[str, int] = {f: 0 for f in FAMILIES}
    n_fail = n_unattr = 0
    for src, kind, _lane in progs:
        lines = src.splitlines()
        in_match = match_depth_lines(src)
        for i, line in enumerate(lines):
            for fam in families_on(line, in_match[i] if i < len(in_match) else False):
                exposed[fam] += 1
        _, diag, code = run_program(src, kind, tmp)
        if code != 0 and diag:
            n_fail += 1
            ln_no = 0
            m = re.match(r"(\d+):", diag.get("at", "") or "")
            if m:
                ln_no = int(m.group(1))
            if 1 <= ln_no <= len(lines):
                fams = families_on(lines[ln_no - 1],
                                   in_match[ln_no - 1] if ln_no - 1 < len(in_match) else False)
                if fams:
                    for fam in fams:
                        attributed[fam] += 1
                else:
                    n_unattr += 1
            else:
                n_unattr += 1
    print(f"programs: {len(progs)}; toolchain failures: {n_fail} "
          f"(unattributable: {n_unattr})\n")
    print(f"{'family':18s}{'class':12s}{'exposed':>9s}{'fails':>7s}{'rate%':>8s}")
    cls_exp: dict[str, int] = {}
    cls_att: dict[str, int] = {}
    for fam, (cls, _) in FAMILIES.items():
        e, a = exposed[fam], attributed[fam]
        cls_exp[cls] = cls_exp.get(cls, 0) + e
        cls_att[cls] = cls_att.get(cls, 0) + a
        print(f"{fam:18s}{cls:12s}{e:>9d}{a:>7d}{(100*a/e if e else 0):>8.2f}")
    print()
    rates = {}
    for cls in ("identical", "similar", "alien"):
        e, a = cls_exp.get(cls, 0), cls_att.get(cls, 0)
        rates[cls] = a / e if e else 0
        print(f"CLASS {cls:12s} exposed {e:>6d}  fails {a:>4d}  rate {100*rates[cls]:.2f}%")
    pred = rates["similar"] > rates["alien"] and rates["similar"] > rates["identical"]
    sep = max(rates.values()) / max(min(rates.values()), 1e-9)
    print(f"\nfrozen prediction (similar > alien AND similar > identical): "
          f"{'HOLDS' if pred else 'BROKEN'}")
    print(f"class separation max/min: {sep:.1f}x "
          f"({'sufficient' if sep >= 2 else 'INSUFFICIENT (<2x refutation)'})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
