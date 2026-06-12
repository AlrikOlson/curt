#!/usr/bin/env python3
"""synth-v2 — tiered-verification batch synthesis.

Tier A: data-pinned templates whose generators COMPUTE the expected
stdout in Python (differential verification, transpiled-corpus trust).
Tier B: freer single-feature templates (from synth.py) plus composed
two/three-feature tasks, verified by k=4 OUTPUT CONSENSUS: outputs are
clustered with the numeric-normalized comparator (norm_eq); the largest
cluster must have >= 2 members and strictly beat any rival.

One k-cluster yields three assets:
  pairs    — every distinct verified member of the winning cluster
             (the fewest-o200k-tokens member is winner-flagged)
  prefs    — (instruction, winner, longest loser) DPO pairs
  repairs  — rejected attempts get ONE repair turn carrying exactly the
             agent-visible context (program + one-line JSON diagnostic);
             verified fixes become (broken, diagnostic, fixed) triples

Main rounds run on the Batch API (50% off). Stages:
  python tools/py2curt/synth2.py submit    # build instructions, submit batch
  python tools/py2curt/synth2.py poll      # wait + download results
  python tools/py2curt/synth2.py filter    # verify, cluster, select; submit repair batch
  python tools/py2curt/synth2.py finish    # poll repair, emit artifacts + report
State lives under data/external/synth2/ (gitignored).
"""

import gzip
import hashlib
import json
import pathlib
import random
import re
import subprocess
import sys
import tempfile
import time
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "data" / "py2curt"
STATE = ROOT / "data" / "external" / "synth2"
MODEL = "claude-sonnet-4-6"
K = 4
TEMP = 0.9

sys.path.insert(0, str(ROOT / "tools" / "py2curt"))
from pipeline import norm_eq  # noqa: E402
from synth import (  # noqa: E402
    FEATURE_RX, SHEET, SYSTEM_SUFFIX, api_key, extract_code, run_curt,
    templates as tier_b_template, ITEMS, KEYS, WORDS,
)

import tiktoken  # noqa: E402

ENC = tiktoken.get_encoding("o200k_base")


# ---------- Tier A: data-pinned templates with computed oracles ----------

def tier_a(rng, i):
    """Returns (feature, instruction, fixture, expected_stdout)."""
    kind = i % 4
    if kind == 0:
        # pinned map: lookups + missing-key rescue + size
        ks = rng.sample(WORDS, 4)
        vs = [rng.randint(2, 99) for _ in ks]
        m = dict(zip(ks, vs))
        lit = ", ".join(f'"{k}": {v}' for k, v in m.items())
        instr = (f'Define exactly this map literal: {{{lit}}}. Print the value for '
                 f'key "{ks[1]}", then the value for key "missing" with fallback 0, '
                 f"then the sum of all values via the pairs verb, each on its own line.")
        expected = f"{m[ks[1]]}\n0\n{sum(vs)}\n"
        return "maplit", instr, None, expected
    if kind == 1:
        # pinned fs aggregation
        names = rng.sample(WORDS, 5)
        nums = [rng.randint(1, 99) for _ in names]
        fixture = "\n".join(f"{n} {v}" for n, v in zip(names, nums))
        best = names[nums.index(max(nums))]
        instr = ('Read the file "data.txt" containing lines of the form "name number". '
                 "Print the line count, the sum of the numbers, and the name on the "
                 "line with the largest number, each on its own line.")
        expected = f"{len(names)}\n{sum(nums)}\n{best}\n"
        return "fs", instr, fixture, expected
    if kind == 2:
        # pinned parse-with-rescue sequence
        good1, good2 = rng.randint(2, 99), rng.randint(2, 99)
        bad = rng.choice(WORDS)
        fb = rng.randint(0, 9)
        instr = (f'Parse the strings "{good1}", "{bad}", and "{good2}" with .int and '
                 f"print each result with rescue fallback {fb}, one per line.")
        expected = f"{good1}\n{fb}\n{good2}\n"
        return "rescue", instr, None, expected
    # pinned pipeline fold
    xs = [rng.randint(1, 30) for _ in range(rng.randint(6, 9))]
    thr = rng.randint(5, 15)
    kept = [x for x in xs if x > thr]
    instr = (f"Given exactly the list {xs}, print the count of values strictly "
             f"greater than {thr}, then the sum of their squares, each on its own line.")
    expected = f"{len(kept)}\n{sum(x * x for x in kept)}\n"
    return "pipeline", instr, None, expected


# ---------- Tier B compositions ----------

def composed(rng, i, n_features):
    feats = rng.sample(["maplit", "rescue", "match_err", "pipeline", "fs", "numjoin"], n_features)
    it, w1, w2 = rng.choice(ITEMS), rng.choice(WORDS), rng.choice(WORDS)
    a, b = rng.randint(2, 9), rng.randint(10, 99)
    parts, fixture = [], None
    for f in feats:
        if f == "maplit":
            parts.append(f'build a map literal of {it} with string keys including "{w1}"')
        elif f == "rescue":
            parts.append(f"print a missing-key or failed-parse lookup rescued to {a}")
        elif f == "match_err":
            parts.append(f'use match with an err arm on a function that rejects values over {b} with err "limit"')
        elif f == "pipeline":
            parts.append(f"use one multi-stage pipeline (keep + map at minimum) over a list you define")
        elif f == "fs":
            lines = "\n".join(f"{rng.choice(WORDS)} {rng.randint(1, 99)}" for _ in range(5))
            fixture = lines
            parts.append('read "data.txt" (lines of "name number") and aggregate the numbers')
        elif f == "numjoin":
            parts.append(f"mix int and float arithmetic (e.g. {a} + {b}.5) in a printed result")
    instr = ("Write one self-contained program that does ALL of the following, printing "
             "results on separate lines: " + "; ".join(parts) + ".")
    return feats, instr, fixture


# ---------- batch client ----------

def api(path, payload=None, method="POST"):
    req = urllib.request.Request(
        f"https://api.anthropic.com{path}",
        data=json.dumps(payload).encode() if payload is not None else None,
        headers={"x-api-key": api_key(), "anthropic-version": "2023-06-01",
                 "content-type": "application/json"},
        method=method,
    )
    with urllib.request.urlopen(req, timeout=300) as r:
        return json.loads(r.read())


def fetch_results(batch_id):
    req = urllib.request.Request(
        f"https://api.anthropic.com/v1/messages/batches/{batch_id}/results",
        headers={"x-api-key": api_key(), "anthropic-version": "2023-06-01"},
    )
    with urllib.request.urlopen(req, timeout=600) as r:
        return [json.loads(l) for l in r.read().decode().splitlines() if l.strip()]


def make_request(custom_id, user_text, max_tokens=900):
    return {
        "custom_id": custom_id,
        "params": {
            "model": MODEL,
            "max_tokens": max_tokens,
            "temperature": TEMP,
            "system": [{"type": "text", "text": SHEET + SYSTEM_SUFFIX,
                        "cache_control": {"type": "ephemeral"}}],
            "messages": [{"role": "user", "content": user_text}],
        },
    }


def submit_batch(requests, name):
    ids = []
    for i in range(0, len(requests), 10000):
        resp = api("/v1/messages/batches", {"requests": requests[i:i + 10000]})
        ids.append(resp["id"])
    (STATE / f"{name}.batchids").write_text("\n".join(ids))
    print(f"{name}: submitted {len(requests)} requests in {len(ids)} batch(es): {ids}")


def poll_batches(name):
    ids = (STATE / f"{name}.batchids").read_text().split()
    results = []
    for bid in ids:
        while True:
            st = api(f"/v1/messages/batches/{bid}", method="GET")
            if st["processing_status"] == "ended":
                break
            print(f"{bid}: {st['processing_status']} "
                  f"({st['request_counts']})", flush=True)
            time.sleep(30)
        results.extend(fetch_results(bid))
    path = STATE / f"{name}.results.jsonl"
    with path.open("w") as f:
        for r in results:
            f.write(json.dumps(r) + "\n")
    print(f"{name}: {len(results)} results -> {path}")


# ---------- verification (per sample) ----------

def verify_sample(features, code, fixture):
    for f in features:
        if f in FEATURE_RX and not FEATURE_RX[f].search(code):
            return None, f"feature-absent:{f}"
    out, ok = run_curt(["check", "-"], code)
    if not ok:
        return None, "check"
    with tempfile.TemporaryDirectory() as d:
        if fixture is not None:
            (pathlib.Path(d) / "data.txt").write_text(fixture + "\n")
        args = ["run", "-", "--fs"] if fixture is not None else ["run", "-"]
        out1, ok1 = run_curt(args, code, cwd=d)
        out2, ok2 = run_curt(args, code, cwd=d)
    if not (ok1 and ok2):
        return None, "runtime"
    if out1 != out2:
        return None, "nondeterministic"
    if not out1.strip():
        return None, "empty-output"
    return out1, None


def check_diag(code):
    r = subprocess.run([str(ROOT / "target/release/curt"), "check", "-"],
                       input=code, capture_output=True, text=True, timeout=20)
    return r.returncode == 0, (r.stderr or r.stdout).strip().splitlines()[-1] if (r.stderr or r.stdout).strip() else ""


def run_diag(code, fixture):
    with tempfile.TemporaryDirectory() as d:
        if fixture is not None:
            (pathlib.Path(d) / "data.txt").write_text(fixture + "\n")
        args = ["run", "-", "--fs"] if fixture is not None else ["run", "-"]
        r = subprocess.run([str(ROOT / "target/release/curt"), *args],
                           input=code, capture_output=True, text=True, timeout=20, cwd=d)
    return (r.stderr or "").strip().splitlines()[-1] if (r.stderr or "").strip() else ""


def tok(code):
    return len(ENC.encode(code))


# ---------- instruction build ----------

def build_instructions():
    """Deterministic instruction set: [{iid, tier, features, instruction, fixture, expected}]."""
    out = []
    rng = random.Random(20260612)
    # Tier A: 480 oracle-verified
    for i in range(480):
        f, instr, fixture, expected = tier_a(rng, i)
        out.append({"iid": f"a{i:04d}", "tier": "A", "features": [f],
                    "instruction": instr, "fixture": fixture, "expected": expected})
    # Tier B singles: 495 across families (reuse v1 templates)
    fams = list(FEATURE_RX)
    for i in range(495):
        fam = fams[i % len(fams)]
        frng = random.Random(hash((fam, i)) % (2**32))
        instr = tier_b_template(fam, frng, i)
        fixture = None
        if "||FIXTURE||" in instr:
            instr, fixture = instr.split("||FIXTURE||")
        out.append({"iid": f"b{i:04d}", "tier": "B", "features": [fam],
                    "instruction": instr, "fixture": fixture, "expected": None})
    # Compositions: 375 two-feature + 150 three-feature (25% / 10% of 1500)
    for i in range(375):
        feats, instr, fixture = composed(rng, i, 2)
        out.append({"iid": f"c2_{i:04d}", "tier": "B", "features": feats,
                    "instruction": instr, "fixture": fixture, "expected": None})
    for i in range(150):
        feats, instr, fixture = composed(rng, i, 3)
        out.append({"iid": f"c3_{i:04d}", "tier": "B", "features": feats,
                    "instruction": instr, "fixture": fixture, "expected": None})
    return out


# ---------- stages ----------

def stage_submit():
    STATE.mkdir(parents=True, exist_ok=True)
    instructions = build_instructions()
    (STATE / "instructions.json").write_text(json.dumps(instructions))
    reqs = [make_request(f"{ins['iid']}__s{s}", ins["instruction"])
            for ins in instructions for s in range(K)]
    submit_batch(reqs, "main")
    print(f"{len(instructions)} instructions x k={K} = {len(reqs)} requests")


def stage_poll():
    poll_batches("main")


def stage_filter():
    instructions = {i["iid"]: i for i in json.loads((STATE / "instructions.json").read_text())}
    results = [json.loads(l) for l in (STATE / "main.results.jsonl").open()]
    by_iid = {}
    usage = {"in": 0, "cached": 0, "out": 0}
    for r in results:
        if r["result"]["type"] != "succeeded":
            continue
        msg = r["result"]["message"]
        u = msg.get("usage", {})
        usage["in"] += u.get("input_tokens", 0)
        usage["cached"] += u.get("cache_read_input_tokens", 0)
        usage["out"] += u.get("output_tokens", 0)
        iid = r["custom_id"].rsplit("__s", 1)[0]
        by_iid.setdefault(iid, []).append(extract_code(msg["content"][0]["text"]))

    clusters_out, rejects, stats = [], [], {"consensus_fail": 0, "oracle_fail": 0}
    for iid, codes in by_iid.items():
        ins = instructions[iid]
        verified = []
        for code in codes:
            stdout, why = verify_sample(ins["features"], code, ins["fixture"])
            if why:
                rejects.append({"iid": iid, "code": code, "why": why,
                                "fixture": ins["fixture"]})
            else:
                verified.append((code, stdout))
        if not verified:
            continue
        if ins["tier"] == "A":
            good = [(c, o) for c, o in verified if norm_eq(o, ins["expected"])]
            for c, o in verified:
                if not norm_eq(o, ins["expected"]):
                    stats["oracle_fail"] += 1
                    rejects.append({"iid": iid, "code": c, "why": "oracle-mismatch",
                                    "fixture": ins["fixture"]})
            if not good:
                continue
            cluster = good
        else:
            groups = []
            for c, o in verified:
                for g in groups:
                    if norm_eq(o, g[0][1]):
                        g.append((c, o))
                        break
                else:
                    groups.append([(c, o)])
            groups.sort(key=len, reverse=True)
            if len(groups[0]) < 2 or (len(groups) > 1 and len(groups[1]) == len(groups[0])):
                stats["consensus_fail"] += 1
                continue
            cluster = groups[0]
        # dedup within cluster, pick winner by token count
        seen, distinct = set(), []
        for c, o in cluster:
            h = hashlib.sha256(c.encode()).hexdigest()
            if h not in seen:
                seen.add(h)
                distinct.append(c)
        distinct.sort(key=tok)
        clusters_out.append({"iid": iid, "tier": ins["tier"], "features": ins["features"],
                             "instruction": ins["instruction"], "fixture": ins["fixture"],
                             "members": distinct})
    (STATE / "clusters.json").write_text(json.dumps(clusters_out))
    (STATE / "rejects.json").write_text(json.dumps(rejects))
    (STATE / "stats.json").write_text(json.dumps({**stats, **usage}))
    print(f"instructions answered: {len(by_iid)}  clusters kept: {len(clusters_out)}  "
          f"rejected samples: {len(rejects)}  consensus_fail: {stats['consensus_fail']}  "
          f"oracle_fail: {stats['oracle_fail']}")
    # repair batch: one turn, agent-visible context only. The custom_id
    # indexes into the DIAGNOSABLE sublist (order-stable via rejects.json).
    for rej in rejects:
        if rej["why"].startswith("feature-absent"):
            continue  # not a diagnosable program error
        ok, diag = check_diag(rej["code"])
        if ok or not diag:
            diag = run_diag(rej["code"], rej["fixture"])
        if diag:
            rej["diag"] = diag
    (STATE / "rejects.json").write_text(json.dumps(rejects))
    diagnosable = [r for r in rejects if r.get("diag")]
    reqs = [
        make_request(
            f"rep{j:05d}",
            f"This curt program fails:\n```curt\n{rej['code']}```\n"
            f"Toolchain diagnostic: {rej['diag']}\nReply with ONLY the fixed program.",
        )
        for j, rej in enumerate(diagnosable)
    ]
    submit_batch(reqs, "repair")


def stage_finish():
    poll_batches("repair")
    instructions = {i["iid"]: i for i in json.loads((STATE / "instructions.json").read_text())}
    clusters = json.loads((STATE / "clusters.json").read_text())
    rejects = json.loads((STATE / "rejects.json").read_text())
    stats = json.loads((STATE / "stats.json").read_text())
    rep_results = [json.loads(l) for l in (STATE / "repair.results.jsonl").open()]
    rep_usage = {"in": 0, "cached": 0, "out": 0}

    pairs, prefs = [], []
    for cl in clusters:
        for rank, code in enumerate(cl["members"]):
            pairs.append({
                "id": f"s2_{cl['iid']}_{rank}", "tier": cl["tier"],
                "features": cl["features"], "instruction": cl["instruction"],
                "curt": code, "fixture": cl["fixture"], "winner": rank == 0,
                "source": "synth2", "model": MODEL,
            })
        if len(cl["members"]) >= 2 and tok(cl["members"][-1]) > tok(cl["members"][0]):
            prefs.append({
                "id": f"p2_{cl['iid']}", "instruction": cl["instruction"],
                "chosen": cl["members"][0], "rejected": cl["members"][-1],
                "chosen_tokens": tok(cl["members"][0]),
                "rejected_tokens": tok(cl["members"][-1]),
                "tier": cl["tier"], "source": "synth2",
            })

    diagnosable = [r for r in rejects if r.get("diag")]
    repairs, rep_fail = [], 0
    for r in rep_results:
        if r["result"]["type"] != "succeeded":
            rep_fail += 1
            continue
        u = r["result"]["message"].get("usage", {})
        for k in rep_usage:
            key = {"in": "input_tokens", "cached": "cache_read_input_tokens",
                   "out": "output_tokens"}[k]
            rep_usage[k] += u.get(key, 0)
        j = int(r["custom_id"][3:])
        rej = diagnosable[j] if j < len(diagnosable) else None
        if rej is None:
            continue
        fixed = extract_code(r["result"]["message"]["content"][0]["text"])
        ins = instructions[rej["iid"]]
        stdout, why = verify_sample(ins["features"], fixed, rej["fixture"])
        if why or (ins["expected"] and not norm_eq(stdout, ins["expected"])):
            rep_fail += 1
            continue
        repairs.append({
            "id": f"r2_{r['custom_id']}", "broken": rej["code"], "diagnostic": rej["diag"],
            "fixed": fixed, "error_class": rej["why"],
            "instruction": ins["instruction"], "source": "synth2",
        })

    # splits: every 10th item per artifact -> eval
    for i, p in enumerate(pairs):
        p["split"] = "eval" if i % 10 == 9 else "train"
    for arts, name in ((pairs, "pairs-synth2"), (prefs, "prefs-density"), (repairs, "repairs")):
        with (OUT / f"{name}.jsonl.gz").open("wb") as raw, \
                gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as f:
            for row in arts:
                f.write((json.dumps(row, sort_keys=True) + "\n").encode())

    # cost at batch rates: in $1.5/M, cached $0.15/M, out $7.5/M
    cost = ((stats["in"] + rep_usage["in"]) / 1e6 * 1.5
            + (stats["cached"] + rep_usage["cached"]) / 1e6 * 0.15
            + (stats["out"] + rep_usage["out"]) / 1e6 * 7.5)
    n_tier_a = sum(1 for p in pairs if p["tier"] == "A")
    comp2 = sum(1 for p in pairs if p["id"].startswith("s2_c2"))
    comp3 = sum(1 for p in pairs if p["id"].startswith("s2_c3"))
    with (OUT / "REPORT-synth2.md").open("w") as f:
        f.write("# synth-v2 pipeline report\n\n")
        f.write("Tiered verification: Tier A pairs match a template-computed expected\n")
        f.write("output (differential verification); Tier B pairs carry k-sample output\n")
        f.write("consensus (k=4, numeric-normalized clustering, strict majority).\n")
        f.write("Winners are the fewest-o200k-token members of their clusters.\n\n")
        f.write(f"- pairs: {len(pairs)} ({n_tier_a} Tier A / {len(pairs) - n_tier_a} Tier B; "
                f"{comp2} from two-feature + {comp3} from three-feature compositions; "
                f"{sum(1 for p in pairs if p['split'] == 'eval')} held out)\n")
        f.write(f"- DPO preference pairs: {len(prefs)}\n")
        f.write(f"- repair triples: {len(repairs)} (repair attempts that failed "
                f"re-verification: {rep_fail})\n")
        f.write(f"- consensus failures (ambiguity signal): {stats['consensus_fail']}\n")
        f.write(f"- oracle mismatches (deterministic-but-wrong, the class v1 could not "
                f"catch): {stats['oracle_fail']}\n")
        f.write(f"- estimated batch-rate cost: ${cost:.2f} (model {MODEL}, k={K}, "
                f"temp {TEMP})\n")
    print(f"pairs {len(pairs)} (A:{n_tier_a})  prefs {len(prefs)}  repairs {len(repairs)}  "
          f"cost ${cost:.2f}")
    print(f"wrote pairs-synth2 / prefs-density / repairs + REPORT-synth2.md")


if __name__ == "__main__":
    stage = sys.argv[1] if len(sys.argv) > 1 else ""
    {"submit": stage_submit, "poll": stage_poll,
     "filter": stage_filter, "finish": stage_finish}[stage]()
