#!/usr/bin/env python3
"""Real-source data pipeline: MBPP / HumanEval -> verified (instruction, curt) pairs.

Reuses pipeline.process() — the same oracle -> transpile -> check -> run ->
verify -> dense -> fmt spine that produced the generated corpus — over the
adapted real-Python seeds from real_sources.gen(). Emits
data/py2curt/pairs-real.jsonl.gz (with source + split fields) and
REPORT-real.md with per-source yield and the full rejection taxonomy.

Usage:
  .ci-venv/bin/python tools/py2curt/fetch_real.py      # once
  .ci-venv/bin/python tools/py2curt/pipeline_real.py
"""

import gzip
import json
import pathlib
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / "data" / "py2curt"

sys.path.insert(0, str(HERE))
from pipeline import process  # noqa: E402
from real_sources import gen  # noqa: E402


def main():
    items = gen()
    seeds = [s for s in items if s["status"] == "seed"]
    adapter_rejects = [s for s in items if s["status"] != "seed"]

    with ThreadPoolExecutor(max_workers=8) as ex:
        results = list(ex.map(process, seeds))

    rows = []
    for seed, res in zip(seeds, results):
        res["source"] = seed["family"]
        res["split"] = seed["params"]["split"]
        rows.append(res)

    ok = [r for r in rows if r["status"] == "ok"]
    rejects = [{"why": r["why"], "source": r["family"]} for r in rows if r["status"] != "ok"]
    rejects += [{"why": s["why"], "source": s["id"].split("_")[0]} for s in adapter_rejects]

    taxonomy = {}
    for r in rejects:
        taxonomy[r["why"]] = taxonomy.get(r["why"], 0) + 1
    per_source = {}
    for s in ("mbpp", "humaneval"):
        total = sum(1 for i in items if i.get("family") == s or i["id"].startswith(s))
        kept = sum(1 for r in ok if r["source"] == s)
        per_source[s] = (kept, total)

    n_total = len(items)
    print(f"problems: {n_total}  verified pairs: {len(ok)} "
          f"({100 * len(ok) / n_total:.1f}% yield)")
    for s, (kept, total) in per_source.items():
        print(f"  {s:<10} {kept}/{total}")
    for why, n in sorted(taxonomy.items(), key=lambda kv: -kv[1])[:20]:
        print(f"  {why:<32} {n}")

    OUT.mkdir(parents=True, exist_ok=True)
    # mtime=0 keeps the archive byte-identical across reruns
    with (OUT / "pairs-real.jsonl.gz").open("wb") as raw, \
            gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as f:
        for r in ok:
            f.write((json.dumps(r, sort_keys=True) + "\n").encode())
    with (OUT / "REPORT-real.md").open("w") as f:
        f.write("# py2curt real-source pipeline report\n\n")
        f.write("Real human-written Python (MBPP, HumanEval) adapted to stdout\n")
        f.write("programs and run through the same verification spine as the\n")
        f.write("generated corpus. High rejection is expected: the transpiler's\n")
        f.write("subset was defined by the seed generator, and wild Python uses\n")
        f.write("constructs outside it. Every rejection is tagged below.\n\n")
        f.write("Sources: MBPP (CC BY 4.0, google-research), HumanEval (MIT,\n")
        f.write("openai); `mbpp_plus` marks membership in the hand-verified\n")
        f.write("EvalPlus MBPP+ subset (Apache-2.0).\n\n")
        f.write("| source | verified pairs | problems | yield | split |\n|---|---|---|---|---|\n")
        for s, (kept, total) in per_source.items():
            split = "train" if s == "mbpp" else "eval (held out)"
            f.write(f"| {s} | {kept} | {total} | {100 * kept / total:.1f}% | {split} |\n")
        f.write("\n## Rejection taxonomy\n\n| tag | count |\n|---|---|\n")
        for why, n in sorted(taxonomy.items(), key=lambda kv: -kv[1]):
            f.write(f"| {why} | {n} |\n")
        f.write("\n## Triage decisions (first-contact taxonomy)\n\n")
        f.write("Extensions accepted (semantics-exact, measured against this corpus):\n")
        f.write("single-param lambdas; `xs[::-1]` -> `.rev`; subscript aug-assign;\n")
        f.write("variadic `min`/`max` -> list verbs; `list(range(..))` and\n")
        f.write("`list(<comprehension>)` identities; unknown builtins now reject at\n")
        f.write("transpile time with an honest `builtin` tag (previously leaked to\n")
        f.write("the curt checker as `unknown_name`); the adapter lowercases\n")
        f.write("Capitalized function names (curt reserves them for types).\n\n")
        f.write("Skipped by design: imports/modules (`stmt: Import` — stdlib growth\n")
        f.write("is a separate measured-admission decision), dict/set literals,\n")
        f.write("`break`/`continue`, `None`, bitwise operators, tuple loop targets,\n")
        f.write("in-place mutation calls, multi-generator comprehensions, and the\n")
        f.write("checker's numeric strictness (`curt-check` residue).\n")
        f.write("\n## Held-out split\n\n")
        f.write("The split is by SOURCE: every HumanEval pair is `split: eval`\n")
        f.write("and must never be trained on; all MBPP pairs are `split: train`.\n")
        f.write("Splitting by source (not randomly) prevents near-duplicate\n")
        f.write("leakage across the train/eval boundary.\n")
        f.write("\n## Decontamination\n\n")
        f.write("Every pair is scanned against the frozen evaluation suites\n")
        f.write("(token-shingle Jaccard, normalized identifiers/numbers/strings):\n\n")
        f.write("```\n.ci-venv/bin/python tools/py2curt/decontam.py \\\n")
        f.write("  data/py2curt/pairs-real.jsonl.gz data/py2curt/pairs.jsonl.gz\n```\n\n")
        f.write("The scan exits non-zero on any hit at Jaccard >= 0.5.\n")
    print(f"wrote {OUT / 'pairs-real.jsonl.gz'} + REPORT-real.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
