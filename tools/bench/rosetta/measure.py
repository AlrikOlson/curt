#!/usr/bin/env python3
"""RosettaCode token measurement for curt, replicating the methodology of
martinalderson.com/posts/which-programming-languages-are-most-token-efficient/
(2026-01-08): RosettaCode task solutions, GPT-4 tokenizer (cl100k_base — the
post used the Xenova/gpt-4 port of the same vocabulary), per-task token
counts averaged across the task set.

Honest caveats, stated wherever this number is quoted:
- The post's exact task list is not published; this is the SAME METHOD on a
  10-task subset of RosettaCode canon expressible in curt v0.3 today, so the
  number is indicative, not a leaderboard entry. Reference scale from the
  post: J 70, Clojure 109 (best of its 19), C ~2.6x Clojure.
- A+B takes its two integers as CLI arguments (curt's stdin is
  capability-gated); RosettaCode's task statement reads stdin.

Every program is EXECUTED and its stdout verified against a pinned sha256
before being counted: unverified programs are not measured.
"""

import hashlib
import pathlib
import subprocess
import sys

import tiktoken

HERE = pathlib.Path(__file__).parent
CURT = HERE / "../../../target/release/curt"

# (file, argv, sha256 of expected stdout)
TASKS = [
    ("100-doors.curt", [], "e0d6ffbca61566fccf5f4347b44c909a563516bf306aed095e5f62dbc3e6d207"),
    ("99-bottles.curt", [], "931c571de33c43f7573d352430a9723ae37ada9454d22f1bedec110d89e860e9"),
    ("fizzbuzz.curt", [], "f039dc221ad122dda8b7226ad5bc68b8654e9e3a42dcea2b37554cd6f91b56af"),
    ("fibonacci.curt", [], "93a9b2b38d0ff170e51dfa05feafc9832ba25b87d3fabd8e4ffbb0778220cd50"),
    ("factorial.curt", [], "b983c444e57992b7de8b05f37514c746b9c0ac63deb6fcc043b5bf89c2949e81"),
    ("gcd.curt", [], "6e2ae11dad0616f66bbb2b6e6556f580bb987fd911d7132aa6bee2bfc7cc7b52"),
    ("reverse-string.curt", [], "412a982c209db83f4ac076b07da52c9a80cebafa6d8d2c451a8cbdb1383fabcf"),
    ("sum-product.curt", [], "c63dfead33b32f6fa669ca66a85bfbc17ebafab4ed762fdd382ecf2b9587dce1"),
    ("palindrome.curt", [], "acb2b288b9f028830645d94e3a4417e5ffc574a024576d6f69b53d989e9d93ea"),
    ("a-b.curt", ["2", "3"], "f0b5c2c2211c8d67ed15e75e656c7862d086e9245420892a7de62cd9ec582a06"),
]


def main() -> int:
    cl100k = tiktoken.get_encoding("cl100k_base")
    o200k = tiktoken.get_encoding("o200k_base")
    rows = []
    for name, argv, want in TASKS:
        src = (HERE / name).read_text()
        out = subprocess.run(
            [str(CURT), "run", str(HERE / name), *argv],
            capture_output=True, text=True, timeout=60,
        )
        got = hashlib.sha256(out.stdout.encode()).hexdigest()
        if out.returncode != 0 or got != want:
            print(f"FAIL {name}: exit={out.returncode} sha256={got[:16]}…")
            return 1
        rows.append((name, len(cl100k.encode(src)), len(o200k.encode(src))))

    print(f"{'task':22s} {'cl100k':>7s} {'o200k':>6s}   (all outputs sha256-verified)")
    for name, c, o in rows:
        print(f"{name:22s} {c:7d} {o:6d}")
    n = len(rows)
    ac = sum(c for _, c, _ in rows) / n
    ao = sum(o for _, _, o in rows) / n
    print(f"{'AVERAGE':22s} {ac:7.1f} {ao:6.1f}   over n={n} tasks")
    print("reference scale (the post, cl100k, its own 19-language task set):")
    print("  J 70 | Clojure 109 | ... | C ~2.6x Clojure")
    return 0


if __name__ == "__main__":
    sys.exit(main())
