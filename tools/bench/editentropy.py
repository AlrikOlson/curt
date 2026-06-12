#!/usr/bin/env python3
"""hx2 — edit-entropy mediation (pre-registered, think:168).

THEORY (think:139): a diagnostic's repair power is how far it collapses the
model's plausible-edit distribution — not its information content or token
count. CLAIM: repair success is mediated by edit diversity; arm identity
adds no predictive power once diversity is controlled.

FROZEN DESIGN (committed before any API call):
  subset    10 errors from tourney/corpus.jsonl, in-file order: first
            5 type_mismatch, 2 expected, 2 unknown_name, 1 unknown_field
            (runtime excluded — static-repair theory)
  arms      A (stored prose-hint diag) / B (typed steelman) / D (typed +
            oracle replacement payload), rendered by diag_tourney verbatim
  sampling  k=5 per (error, arm), haiku, temp 1.0, SINGLE turn, the
            tournament user prompt; frozen to tourney/hx2_samples.jsonl
  signature edit signature of a sample = set of (line_idx, new_line) pairs
            from a line diff of extracted program vs broken (fmt-normalized
            when parseable; parse-failures keep raw text — the plausible-
            edit distribution INCLUDES failures)
  diversity cell d = mean pairwise Jaccard DISTANCE among the k signatures
  success   cell s = fraction of k samples running clean + oracle match
  analysis  Pearson+Spearman corr(s,d) over 30 cells; OLS R²[s~d],
            R²[s~arm], R²[s~d+arm].
  verdict   CONFIRMED iff corr ≤ -0.5 AND ΔR²(arm|d) < 0.05
            REFUTED   iff corr ≥ -0.3 OR  ΔR²(arm|d) ≥ 0.15
            otherwise inconclusive — published as such.

Usage: editentropy.py sample | run | report
"""

import json
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from diag_tourney import REPAIRS, TOURNEY, render, replacement_payload  # noqa: E402
from loop import CURT, MODELS, PRICES, SYSTEMS, api_key, call_model, extract_code, norm_eq  # noqa: E402

K = 5
ARMS = ["A", "B", "D"]
MODEL = "haiku"
STRATA = [("type_mismatch", 5), ("expected", 2), ("unknown_name", 2), ("unknown_field", 1)]
SAMPLES = TOURNEY / "hx2_samples.jsonl"


def subset() -> list[dict]:
    rows = [json.loads(ln) for ln in (TOURNEY / "corpus.jsonl").open()]
    out = []
    for err, n in STRATA:
        out.extend([r for r in rows if r["diag"]["err"] == err][:n])
    assert len(out) == 10, len(out)
    return out


def cmd_run(_args) -> int:
    key = api_key()
    items = subset()
    total = 0.0
    with SAMPLES.open("w") as f:
        for item in items:
            for arm in ARMS:
                payload = None
                if arm == "D":
                    payload, _ = replacement_payload(item["broken"], item["fixed"])
                diag_text = render(arm, item["diag"], item["diag"].get("fix", ""), payload)
                user = (f"Task: {item['instruction']}\n\nThis curt program is "
                        f"broken:\n```curt\n{item['broken']}```\n\nThe toolchain "
                        f"reports:\n{diag_text}\n\nReply with ONLY the corrected "
                        f"curt program in a code block.")
                for s in range(K):
                    reply, usage, _ = call_model(MODELS[MODEL], SYSTEMS["curt"],
                                                 [{"role": "user", "content": user}], key)
                    cost = (usage.get("input_tokens", 0) * PRICES[MODELS[MODEL]][0]
                            + usage.get("cache_creation_input_tokens", 0) * 1.25 * PRICES[MODELS[MODEL]][0]
                            + usage.get("cache_read_input_tokens", 0) * 0.1 * PRICES[MODELS[MODEL]][0]
                            + usage.get("output_tokens", 0) * PRICES[MODELS[MODEL]][1]) / 1e6
                    total += cost
                    f.write(json.dumps({"id": item["id"], "arm": arm, "k": s,
                                        "diag_tok_text": diag_text,
                                        "program": extract_code(reply, "curt")}) + "\n")
                    print(f"{item['id']} {arm} k{s} ${cost:.4f}", flush=True)
    print(f"total spend ${total:.4f}")
    return 0


def normalize(src: str, tmp: pathlib.Path) -> str:
    """fmt-canonicalize when the program parses; raw otherwise."""
    tmp.write_text(src)
    p = subprocess.run([str(CURT), "fmt", str(tmp)], capture_output=True, text=True, timeout=20)
    return p.stdout if p.returncode == 0 and p.stdout else src


def signature(broken: str, program: str) -> frozenset:
    bl = broken.splitlines()
    pl = program.splitlines()
    import difflib
    sig = set()
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, bl, pl).get_opcodes():
        if tag != "equal":
            sig.add((i1, "\n".join(pl[j1:j2])))
    return frozenset(sig)


def jaccard_dist(a: frozenset, b: frozenset) -> float:
    if not a and not b:
        return 0.0
    return 1.0 - len(a & b) / len(a | b)


def run_oracle(program: str, oracle: str, tmp: pathlib.Path) -> bool:
    tmp.write_text(program)
    try:
        c = subprocess.run([str(CURT), "check", str(tmp)], capture_output=True, text=True, timeout=20)
        if c.returncode != 0:
            return False
        p = subprocess.run([str(CURT), "run", str(tmp)], capture_output=True, text=True, timeout=20)
    except subprocess.TimeoutExpired:
        return False
    return p.returncode == 0 and norm_eq(p.stdout, oracle)


def ols_r2(y: list[float], xs: list[list[float]]) -> float:
    """R² of y ~ [1, xs...] via normal equations (tiny n, pure python)."""
    n = len(y)
    cols = [[1.0] * n] + xs
    k = len(cols)
    a = [[sum(cols[i][t] * cols[j][t] for t in range(n)) for j in range(k)] for i in range(k)]
    b = [sum(cols[i][t] * y[t] for t in range(n)) for i in range(k)]
    # gaussian elimination
    for i in range(k):
        piv = max(range(i, k), key=lambda r: abs(a[r][i]))
        a[i], a[piv] = a[piv], a[i]
        b[i], b[piv] = b[piv], b[i]
        if abs(a[i][i]) < 1e-12:
            return 0.0
        for r in range(k):
            if r != i:
                f = a[r][i] / a[i][i]
                a[r] = [a[r][c] - f * a[i][c] for c in range(k)]
                b[r] -= f * b[i]
    beta = [b[i] / a[i][i] for i in range(k)]
    yhat = [sum(beta[j] * cols[j][t] for j in range(k)) for t in range(n)]
    ybar = sum(y) / n
    ss_res = sum((y[t] - yhat[t]) ** 2 for t in range(n))
    ss_tot = sum((y[t] - ybar) ** 2 for t in range(n))
    return 1.0 - ss_res / ss_tot if ss_tot > 0 else 0.0


def pearson(x: list[float], y: list[float]) -> float:
    n = len(x)
    mx, my = sum(x) / n, sum(y) / n
    num = sum((a - mx) * (b - my) for a, b in zip(x, y))
    dx = sum((a - mx) ** 2 for a in x) ** 0.5
    dy = sum((b - my) ** 2 for b in y) ** 0.5
    return num / (dx * dy) if dx > 0 and dy > 0 else 0.0


def spearman(x: list[float], y: list[float]) -> float:
    def ranks(v):
        order = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0] * len(v)
        i = 0
        while i < len(order):
            j = i
            while j + 1 < len(order) and v[order[j + 1]] == v[order[i]]:
                j += 1
            avg = (i + j) / 2 + 1
            for t in range(i, j + 1):
                r[order[t]] = avg
            i = j + 1
        return r
    return pearson(ranks(x), ranks(y))


def cmd_report(_args) -> int:
    items = {r["id"]: r for r in subset()}
    samples = [json.loads(ln) for ln in SAMPLES.open()]
    tmp = pathlib.Path("/tmp/hx2_probe.curt")
    cells = []
    for item_id, item in items.items():
        broken_norm = normalize(item["broken"], tmp)
        for arm in ARMS:
            group = [s for s in samples if s["id"] == item_id and s["arm"] == arm]
            sigs = [signature(broken_norm, normalize(s["program"], tmp)) for s in group]
            pairs = [(i, j) for i in range(len(sigs)) for j in range(i + 1, len(sigs))]
            d = sum(jaccard_dist(sigs[i], sigs[j]) for i, j in pairs) / len(pairs)
            s_frac = sum(run_oracle(s["program"], item["oracle"], tmp) for s in group) / len(group)
            cells.append({"id": item_id, "err": item["diag"]["err"], "arm": arm,
                          "diversity": round(d, 4), "success": s_frac})
    ds = [c["diversity"] for c in cells]
    ss = [c["success"] for c in cells]
    arm_b = [1.0 if c["arm"] == "B" else 0.0 for c in cells]
    arm_d = [1.0 if c["arm"] == "D" else 0.0 for c in cells]
    pe, sp = pearson(ss, ds), spearman(ss, ds)
    r2_d = ols_r2(ss, [ds])
    r2_arm = ols_r2(ss, [arm_b, arm_d])
    r2_both = ols_r2(ss, [ds, arm_b, arm_d])
    delta = r2_both - r2_d
    if pe <= -0.5 and delta < 0.05:
        verdict = "CONFIRMED"
    elif pe >= -0.3 or delta >= 0.15:
        verdict = "REFUTED"
    else:
        verdict = "INCONCLUSIVE"
    print(f"{'id':16s} {'err':14s} arm  diversity success")
    for c in cells:
        print(f"{c['id']:16s} {c['err']:14s} {c['arm']:3s}  {c['diversity']:9.3f} {c['success']:7.2f}")
    print(f"\nper-arm means: " + "  ".join(
        f"{a}: d={sum(c['diversity'] for c in cells if c['arm'] == a)/10:.3f} "
        f"s={sum(c['success'] for c in cells if c['arm'] == a)/10:.2f}" for a in ARMS))
    print(f"pearson(s,d)={pe:+.3f}  spearman={sp:+.3f}")
    print(f"R2[s~d]={r2_d:.3f}  R2[s~arm]={r2_arm:.3f}  R2[s~d+arm]={r2_both:.3f}  "
          f"dR2(arm|d)={delta:.3f}")
    print(f"VERDICT (pre-registered, think:168): {verdict}")
    return 0


def main() -> int:
    import argparse
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("run").set_defaults(fn=cmd_run)
    sub.add_parser("report").set_defaults(fn=cmd_report)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
