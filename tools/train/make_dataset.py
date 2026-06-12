#!/usr/bin/env python3
"""Assemble the curt-coder SFT dataset (finetune-probe chunk).

Sources (all execution-verified upstream; counts re-checked here):
  data/py2curt/pairs.jsonl.gz         transpiled MBPP-family bulk
  data/py2curt/pairs-synth.jsonl.gz   curt-native synthesis v1
  data/py2curt/pairs-synth2.jsonl.gz  curt-native synthesis v2 (oracle-tiered)
  data/py2curt/pairs-real.jsonl.gz    wild-Python transpilation (22.5% yield)

Output (mlx-lm chat format, completion-masked training upstream):
  data/curtcoder/train.jsonl, valid.jsonl   {"messages": [...]}
  data/curtcoder/MANIFEST.json              reproducible counts + split hash

Design decisions (think:158):
- TRAIN system prompt is the SHORT one (the model should know curt from
  weights, not from a 2k-token sheet); the in-context-sheet eval arm keeps
  the CHEATSHEET. The assistant turn is a ```curt block so the existing
  bench extractor works unchanged.
- Valid split: synth2 ships its own train/eval split (respected verbatim);
  the other sources split by id hash (~5%). Family-holdout was tried and
  abandoned — the sources have 13/1/2 template families, so family buckets
  give lumpy 0.4% splits. Consequence, stated honestly: the valid loss
  measures in-distribution fit (same templates, unseen params), NOT
  generalization; generalization is measured on the frozen bench.
- Admission: bulk/synth/real need status == "ok". synth2 has no status —
  tier A rows (differentially verified) are all admitted; tier B
  (consensus-only) admits winners only, the fewest-token cluster members,
  which is also curt's density bias.
- Programs are kept verbatim (already canonical from their generators).
"""

import gzip
import hashlib
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = [
    ("bulk", ROOT / "data" / "py2curt" / "pairs.jsonl.gz"),
    ("synth", ROOT / "data" / "py2curt" / "pairs-synth.jsonl.gz"),
    ("synth2", ROOT / "data" / "py2curt" / "pairs-synth2.jsonl.gz"),
    ("real", ROOT / "data" / "py2curt" / "pairs-real.jsonl.gz"),
]
OUT = ROOT / "data" / "curtcoder"
VALID_PCT = 5  # per-family hash bucket

SYSTEM = "You write curt programs. Reply with ONLY a curt code block — no prose."


def row_messages(r: dict) -> dict:
    code = r["curt"].rstrip("\n")
    return {
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": r["instruction"]},
            {"role": "assistant", "content": f"```curt\n{code}\n```"},
        ]
    }


def id_bucket(rid: str) -> int:
    return int(hashlib.sha256(rid.encode()).hexdigest(), 16) % 100


def admit(tag: str, r: dict) -> bool:
    if tag == "synth2":
        return r.get("tier") == "A" or bool(r.get("winner"))
    return r.get("status") == "ok"


def to_valid(tag: str, r: dict, rid: str) -> bool:
    if tag == "synth2":
        return r.get("split") == "eval"
    return id_bucket(rid) < VALID_PCT


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    train, valid = [], []
    manifest: dict = {"sources": {}, "valid_pct": VALID_PCT, "system": SYSTEM}
    seen_ids: set[str] = set()
    dropped_admit = dropped_dup = 0
    for tag, path in SRC:
        n = 0
        with gzip.open(path, "rt") as f:
            for ln in f:
                r = json.loads(ln)
                if not admit(tag, r):
                    dropped_admit += 1
                    continue
                rid = f"{tag}:{r['id']}"
                if rid in seen_ids:
                    dropped_dup += 1
                    continue
                seen_ids.add(rid)
                (valid if to_valid(tag, r, rid) else train).append(
                    (rid, row_messages(r))
                )
                n += 1
        manifest["sources"][tag] = n
    # deterministic order: by source-qualified id
    train.sort(key=lambda t: t[0])
    valid.sort(key=lambda t: t[0])
    for name, rows in [("train", train), ("valid", valid)]:
        with (OUT / f"{name}.jsonl").open("w") as f:
            for _, m in rows:
                f.write(json.dumps(m, ensure_ascii=False) + "\n")
    manifest["train"] = len(train)
    manifest["valid"] = len(valid)
    manifest["dropped_admit"] = dropped_admit
    manifest["dropped_dup"] = dropped_dup
    h = hashlib.sha256()
    for name in ("train", "valid"):
        h.update((OUT / f"{name}.jsonl").read_bytes())
    manifest["sha256"] = h.hexdigest()
    (OUT / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
