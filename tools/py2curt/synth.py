#!/usr/bin/env python3
"""corpus-synth — toolchain-verified synthesis for the curt-native surface.

We author the instructions (deterministic PRNG templates, 8 feature
families with quotas); the model writes ONLY the program. The toolchain
is the rejection filter:

  extract code -> curt check -> run twice (byte-identical; determinism
  stands in for the missing oracle) -> nonempty stdout -> feature
  PRESENCE check (a maplit task must contain a map literal) ->
  fmt-canonical sha256 dedup

Every accepted pair carries provenance: feature, template id, model,
seed params. Every 10th accepted pair per feature -> eval split.

The API key is read from data/external/anthropic.key (gitignored) or
$ANTHROPIC_API_KEY. The 1,983-token CHEATSHEET is the cached system
prefix (prompt caching cuts repeat input cost ~90%).

Usage:
  .ci-venv/bin/python tools/py2curt/synth.py --smoke   # 3 attempts/feature
  .ci-venv/bin/python tools/py2curt/synth.py           # full quota run
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
from concurrent.futures import ThreadPoolExecutor

ROOT = pathlib.Path(__file__).resolve().parents[2]
CURT = ROOT / "target" / "release" / "curt"
OUT = ROOT / "data" / "py2curt"
MODEL = "claude-sonnet-4-6"
QUOTA = 140          # accepted pairs per feature
MAX_ATTEMPTS = 400   # attempt cap per feature (budget guard)

ITEMS = ["orders", "users", "events", "scores", "files", "tasks", "logs", "items"]
KEYS = ["name", "size", "kind", "status", "level", "city", "team", "code"]
WORDS = ["alpha", "beta", "gamma", "delta", "omega", "kappa", "sigma", "theta"]


def templates(feature, rng, i):
    """One deterministic instruction per (feature, index)."""
    it, k1, k2 = rng.choice(ITEMS), rng.choice(KEYS), rng.choice(KEYS)
    w1, w2 = rng.choice(WORDS), rng.choice(WORDS)
    a, b, n = rng.randint(2, 9), rng.randint(10, 99), rng.randint(3, 7)
    if feature == "maplit":
        return rng.choice([
            f'Define a map literal of {n} {it} with string keys (e.g. "{w1}", "{w2}", ...) and integer counts. Print the value for key "{w1}", then add a new key "{k1}" with value {b}, then print the map size.',
            f'Create a config map literal with keys "{k1}" (an int), "{k2}" (a string), and "limit" ({b}). Print each value on its own line, then print the doubled limit.',
            f'Build a map literal of {n} entries mapping names to scores. Print the sum of all values using the pairs verb, then print the value for a missing key with fallback {a}.',
        ])
    if feature == "match_err":
        return rng.choice([
            f'Define a function classify that takes a value of type int | str and uses match to return "num" doubled-value text for ints and uppercase for strings. Print classify {a} and classify "{w1}".',
            f'Define a function that returns err "too big" when its int argument exceeds {b}, else the argument times {a}. Call it on {b - 5} and {b + 5}; use match with an err pattern to print either the value or the error message.',
            f'Use match on the result of parsing the string "{b}" with .int: print the parsed number plus {a}. Then match on parsing "{w1}" and print "bad input" via an err arm.',
        ])
    if feature == "rescue":
        return rng.choice([
            f'Build a list of {n} integers, index it out of bounds, and print the result rescued with fallback {a}. Then chain two rescues: a missing map key rescued by another missing key rescued by {b}.',
            f'Parse the strings "{a}", "{w1}", and "{b}" with .int, printing each result with a rescue fallback of 0, one per line.',
            f'Create an empty map, then print the value for key "{k1}" rescued to "{w1}", and print a list max on an empty slice rescued to {a}.',
        ])
    if feature == "rawstr":
        return rng.choice([
            f"Bind a raw single-quoted string containing literal braces and backslashes (e.g. a pattern like '\\d+ {{x}}'), print it, print its length, and print it uppercased.",
            f"Use a raw single-quoted string for a path-like value 'C:\\{w1}\\{w2}', print it as-is, then split it on the backslash and print how many parts result.",
            f"Bind one raw '...' string holding the literal text {{not interpolated}} and one normal string that interpolates a variable x = {a}; print both.",
        ])
    if feature == "fs":
        lines = "\n".join(f"{rng.choice(WORDS)} {rng.randint(1, 99)}" for _ in range(n))
        return (
            f'Read the file "data.txt" (it contains lines of the form "name number"). '
            f"Print the number of lines, then the sum of the numeric second fields, "
            f"then the name with the largest number.||FIXTURE||{lines}"
        )
    if feature == "pipeline":
        return rng.choice([
            f"Given a list of {n + 4} integers you define, build one pipeline that keeps values above {a}, squares them, and prints the sum. Then a second pipeline printing the top 2 values.",
            f'Given a list of words you define (mixing "{w1}" and "{w2}" repeats), use counts to build a frequency map and print the count of "{w1}", then print the number of distinct words via pairs.',
            f"Define a list of {n + 3} integers, group them by value modulo {a}, and print how many groups result. Then print the sorted distinct remainders, space-joined, using one pipeline.",
        ])
    if feature == "numjoin":
        return rng.choice([
            f"Compute a running average: start total = 0, add the values {a}, {b}, and {a}.5 in a loop or directly, and print total / 3 (a float). Then print whether {a} equals {a}.0.",
            f"Define rate = {a}.5 and count = {b} (an int). Print count * rate, count + rate, and the integer part of the product via .int, each on its own line.",
            f"Mix ints and floats: x = {a}, y = {b}.25. Print x + y, x * 2, y * 2, and (x < y), one per line.",
        ])
    if feature == "annot":
        return rng.choice([
            f"Declare a signature area :: int int -> int, define area as width times height, and print area {a} {b}. Then bind limit: int = {b} with an annotation and print limit - {a}.",
            f"Write a function scale with signature scale :: float int -> float multiplying its arguments; print scale {a}.5 {n}. Annotate a binding name: str = \"{w1}\" and print it uppercased.",
            f"Define two annotated bindings (count: int = {b}, label: str = \"{w1}\") and a signed function next :: int -> int returning its argument plus 1. Print next count and label.",
        ])
    raise ValueError(feature)


FEATURE_RX = {
    "maplit": re.compile(r'\{\s*"'),
    "match_err": re.compile(r"\bmatch\b"),
    "rescue": re.compile(r"\?"),
    "rawstr": re.compile(r"'[^']*'"),
    "fs": re.compile(r"\bfs\."),
    "pipeline": re.compile(r"\|.*\|"),
    "numjoin": re.compile(r"\d+\.\d+"),
    "annot": re.compile(r"::|:\s*(int|str|float|bool)\s*="),
}

SYSTEM_SUFFIX = (
    "\n\nYou write curt programs. Reply with ONLY a curt code block — no prose. "
    "The program must be self-contained, deterministic, and print its results."
)


def api_key():
    import os
    k = os.environ.get("ANTHROPIC_API_KEY")
    if k:
        return k
    return (ROOT / "data" / "external" / "anthropic.key").read_text().strip()


SHEET = (ROOT / "CHEATSHEET.md").read_text()


def generate(instruction, key):
    body = {
        "model": MODEL,
        "max_tokens": 700,
        "system": [{
            "type": "text",
            "text": SHEET + SYSTEM_SUFFIX,
            "cache_control": {"type": "ephemeral"},
        }],
        "messages": [{"role": "user", "content": instruction}],
    }
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(body).encode(),
        headers={
            "x-api-key": key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        },
    )
    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                data = json.load(r)
            usage = data.get("usage", {})
            return data["content"][0]["text"], usage
        except urllib.error.HTTPError as e:
            if e.code in (429, 500, 529):
                time.sleep(2 ** attempt + 1)
                continue
            raise
        except (TimeoutError, OSError):
            time.sleep(2 ** attempt + 1)
    return None, {}


def extract_code(text):
    m = re.search(r"```(?:curt)?\n(.*?)```", text, re.DOTALL)
    return (m.group(1) if m else text).strip() + "\n"


def run_curt(args, src, cwd=None):
    r = subprocess.run(
        [str(CURT), *args], input=src, capture_output=True, text=True, timeout=20, cwd=cwd
    )
    return (r.stdout, r.returncode == 0)


def verify(feature, code, fixture):
    """Filter chain; returns (why_rejected | None)."""
    if not FEATURE_RX[feature].search(code):
        return "feature-absent"
    _, ok = run_curt(["check", "-"], code)
    if not ok:
        return "check"
    with tempfile.TemporaryDirectory() as d:
        if fixture is not None:
            (pathlib.Path(d) / "data.txt").write_text(fixture + "\n")
        run_args = ["run", "-", "--fs"] if fixture is not None else ["run", "-"]
        out1, ok1 = run_curt(run_args, code, cwd=d)
        out2, ok2 = run_curt(run_args, code, cwd=d)
    if not (ok1 and ok2):
        return "runtime"
    if out1 != out2:
        return "nondeterministic"
    if not out1.strip():
        return "empty-output"
    return None


def canonical(code):
    out, ok = run_curt(["fmt", "-"], code)
    return out if ok else code


def process(job, key, seen, lock):
    feature, idx, instruction, fixture = job
    text, usage = generate(instruction, key)
    if text is None:
        return {"feature": feature, "status": "reject", "why": "api-error", "usage": {}}
    code = extract_code(text)
    why = verify(feature, code, fixture)
    if why:
        return {"feature": feature, "status": "reject", "why": why, "usage": usage}
    # fmt preserves raw '...' spellings since fmt-rawstr (2026-06-12) —
    # the old rawstr exemption (89 erased pairs in the first full run) is
    # no longer needed, but kept harmless: canonical(code) is now a no-op
    # on raw strings either way.
    code = canonical(code)
    digest = hashlib.sha256(code.encode()).hexdigest()
    with lock:
        if digest in seen:
            return {"feature": feature, "status": "reject", "why": "duplicate", "usage": usage}
        seen.add(digest)
    return {
        "id": f"synth_{feature}_{idx:04d}",
        "feature": feature,
        "instruction": instruction.split("||FIXTURE||")[0],
        "curt": code,
        "fixture": fixture,
        "provenance": {"seed": "template", "template_feature": feature,
                       "template_index": idx, "model": MODEL},
        "status": "ok",
        "usage": usage,
    }


def main():
    import threading
    smoke = "--smoke" in sys.argv
    quota = 3 if smoke else QUOTA
    cap = 6 if smoke else MAX_ATTEMPTS
    key = api_key()
    features = list(FEATURE_RX)
    only = None
    if "--only" in sys.argv:
        only = sys.argv[sys.argv.index("--only") + 1]
        features = [only]

    accepted = {f: [] for f in features}
    taxonomy = {}
    tokens = {"in": 0, "cached": 0, "out": 0}
    seen: set = set()
    lock = threading.Lock()

    for feature in features:
        rng = random.Random(hash(feature) % (2**32))
        idx = 0
        while len(accepted[feature]) < quota and idx < cap:
            batch = []
            for _ in range(min(8, cap - idx)):
                instr = templates(feature, rng, idx)
                fixture = None
                if "||FIXTURE||" in instr:
                    instr_text, fixture = instr.split("||FIXTURE||")
                    instr = instr_text + "||FIXTURE||" + fixture
                batch.append((feature, idx, instr, fixture))
                idx += 1
            with ThreadPoolExecutor(max_workers=8) as ex:
                results = list(ex.map(lambda j: process(j, key, seen, lock), batch))
            for r in results:
                u = r.pop("usage", {})
                tokens["in"] += u.get("input_tokens", 0)
                tokens["cached"] += u.get("cache_read_input_tokens", 0)
                tokens["out"] += u.get("output_tokens", 0)
                if r["status"] == "ok":
                    if len(accepted[feature]) < quota:
                        accepted[feature].append(r)
                else:
                    key_w = f"{r['feature']}:{r['why']}"
                    taxonomy[key_w] = taxonomy.get(key_w, 0) + 1
        print(f"{feature:<10} accepted {len(accepted[feature]):>4}  attempts {idx}", flush=True)

    rows = []
    for f in features:
        for j, r in enumerate(accepted[f]):
            r["split"] = "eval" if j % 10 == 9 else "train"
            r["source"] = "synth"
            rows.append(r)

    # cost: sonnet-4-6 $3/M in, $0.30/M cached read, $15/M out
    cost = tokens["in"] / 1e6 * 3 + tokens["cached"] / 1e6 * 0.30 + tokens["out"] / 1e6 * 15
    n_rej = sum(taxonomy.values())
    print(f"\naccepted {len(rows)}  rejected {n_rej} "
          f"({100 * n_rej / max(1, n_rej + len(rows)):.1f}%)  est cost ${cost:.2f}")
    for k, n in sorted(taxonomy.items(), key=lambda kv: -kv[1]):
        print(f"  {k:<28} {n}")

    if smoke:
        return 0

    OUT.mkdir(parents=True, exist_ok=True)
    pairs_path = OUT / "pairs-synth.jsonl.gz"
    if only and pairs_path.exists():
        # surgical regeneration: keep every other feature's existing rows
        kept = [json.loads(l) for l in gzip.open(pairs_path, "rt")
                if json.loads(l)["feature"] != only]
        rows = kept + rows
    with pairs_path.open("wb") as raw, \
            gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as f:
        for r in rows:
            f.write((json.dumps(r, sort_keys=True) + "\n").encode())
    if only:
        with (OUT / "REPORT-synth.md").open("a") as f:
            f.write(f"\n## Correction: `{only}` regenerated\n\n")
            f.write("The first run's fmt canonicalization erased raw `'...'`\n")
            f.write("strings (rewritten to escaped form) AFTER the feature-presence\n")
            f.write(f"check — the family was regenerated with canonicalization\n")
            f.write(f"skipped. New count: {len(accepted[only])} "
                    f"(rejections this round: {sum(taxonomy.values())}; "
                    f"est cost ${cost:.2f}).\n")
        print(f"merged: {len(rows)} total pairs")
        return 0
    by_feature = {f: len(accepted[f]) for f in features}
    rej_by_feature = {}
    for k, n in taxonomy.items():
        f, why = k.split(":", 1)
        rej_by_feature.setdefault(f, {})[why] = n
    with (OUT / "REPORT-synth.md").open("w") as f:
        f.write("# corpus-synth pipeline report\n\n")
        f.write("Toolchain-verified synthesis: deterministic instruction templates\n")
        f.write("(we author the task; the model writes only the program), filtered by\n")
        f.write("check -> double-run determinism -> nonempty output -> feature\n")
        f.write("presence -> canonical-form dedup. No Python oracle exists for these\n")
        f.write("pairs; determinism, the checker, and a manual audit stand in.\n\n")
        f.write(f"Generator: {MODEL}. Estimated API cost: ${cost:.2f} "
                f"(input {tokens['in']:,} + cached {tokens['cached']:,} "
                f"+ output {tokens['out']:,} tokens).\n\n")
        f.write("| feature | accepted | rejections |\n|---|---|---|\n")
        for feat in features:
            rej = ", ".join(f"{w} {n}" for w, n in sorted(
                rej_by_feature.get(feat, {}).items(), key=lambda kv: -kv[1])) or "none"
            f.write(f"| {feat} | {by_feature[feat]} | {rej} |\n")
        f.write(f"\nTotal: {len(rows)} pairs "
                f"({sum(1 for r in rows if r['split'] == 'eval')} held out as eval — "
                f"every 10th accepted pair per feature).\n")
    print(f"wrote {OUT / 'pairs-synth.jsonl.gz'} + REPORT-synth.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
