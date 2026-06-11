#!/usr/bin/env python3
"""gd-b-oss: the OSS constrained-decoding demo.

Two arms against a local llama-server:
  constrained   — curt.gbnf (runtime variant: root ::= stmt nl, so the grammar
                  completes and generation terminates) — EVERY sample must
                  parse via the Rust oracle (`curt parse -`). Gate: 0 errors.
  unconstrained — same prompts, no grammar. Parse rate measured for contrast
                  (the model has zero curt in its weights; this is the
                  Python-drift number the mask exists to eliminate).

Usage:
  tools/grammar/constrained_demo.py --model <gguf> [--n 200] [--n-free 100]
  tools/grammar/constrained_demo.py --server-url http://127.0.0.1:8123 ...

Writes tools/grammar/results.json. Re-runnable.
"""

import argparse
import json
import pathlib
import subprocess
import sys
import time
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CMM = ROOT / "target" / "release" / "curt"

PROMPTS = [
    "Write one statement in the curt programming language that prints a greeting.",
    "Write one curt statement that binds a list of numbers to a name.",
    "Write one curt statement that defines a function adding two numbers.",
    "Write one curt statement that prints the sum of a list.",
    "Write one curt statement using a pipeline to filter a list.",
    "Write one curt statement that defines a record type for a 2D point.",
    "Write one curt statement with a while loop counting down.",
    "Write one curt statement that prints the length of a string.",
    "Write one curt statement that maps a lambda over a list.",
    "Write one curt statement matching on a value.",
]


def runtime_grammar() -> str:
    g = (HERE / "curt.gbnf").read_text()
    # demo variant: one statement per generation so the grammar COMPLETES,
    # and whitespace pruned to single spaces / a bare newline — without
    # this, masked sampling wanders in legal-but-degenerate ws/identifier
    # chains and hits the token cap before completing (a live instance of
    # the masking-distortion caveat from the 2026-06-10 refresh)
    g = g.replace("root ::= start", 'root ::= stmt "\\n"', 1)
    g = g.replace("ws ::= [ \\t]*", 'ws ::= " "?', 1)
    g = g.replace("ws1 ::= [ \\t]+", 'ws1 ::= " "', 1)
    return g



def newline_ids(url: str) -> list:
    req = urllib.request.Request(
        url + "/tokenize",
        data=json.dumps({"content": "\n"}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read()).get("tokens", [])


def chatml(p: str) -> str:
    return (
        "<|im_start|>system\nYou write code in curt, a terse programming "
        "language similar to a pipeline-flavored Python with `=` equations. "
        "Reply with exactly one short line of curt code and nothing else."
        "<|im_end|>\n<|im_start|>user\n" + p + "<|im_end|>\n<|im_start|>assistant\n"
    )


def post(url: str, payload: dict, timeout: int = 300) -> dict:
    req = urllib.request.Request(
        url + "/completion",
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read())


def parses(src: str) -> bool:
    if not src.strip():
        return False
    proc = subprocess.run([str(CMM), "parse", "-"], input=(src.rstrip() + "\n").encode(), capture_output=True, check=False)
    return proc.returncode == 0


def first_line(text: str) -> str:
    for ln in text.splitlines():
        if ln.strip():
            return ln
    return ""


def arm(url: str, n: int, grammar: str | None) -> dict:
    ok, samples, fails = 0, [], []
    # termination pressure: bias the newline token upward — the grammar mask
    # still decides WHAT is legal; bias only reorders within the legal set
    nl_bias = [[tid, 4.0] for tid in newline_ids(url)] if grammar else []
    retries = 0
    completed = 0
    ok_completed = 0
    for i in range(n):
        payload = {
            "prompt": chatml(PROMPTS[i % len(PROMPTS)]),
            "n_predict": 128,
            "temperature": 0.4,
            "seed": i,
            "cache_prompt": False,
        }
        if grammar:
            payload["grammar"] = grammar
            payload["logit_bias"] = nl_bias
        t0 = time.time()
        resp = post(url, payload)
        # token-cap truncation interrupts the grammar before completion —
        # retry once with doubled budget (production-realistic, counted)
        if grammar and not resp.get("content", "").endswith("\n"):
            retries += 1
            payload2 = dict(payload, n_predict=payload["n_predict"] * 2)
            resp = post(url, payload2)
        text = resp.get("content", "")
        # grammar completion is STRUCTURAL: root ::= stmt "\n" — an output
        # without the trailing newline cannot be grammar-accepted, even if
        # the engine emitted EOS (llama.cpp permits the EOS escape mid-
        # grammar — measured here: `record 2-point == {` stopped at eos)
        truncated = not text.endswith("\n")
        completed += not truncated
        dt = time.time() - t0
        if dt > 30:
            print(f"  slow sample {i}: {dt:.0f}s", file=sys.stderr)
        snippet = first_line(text)
        good = parses(snippet)
        ok += good
        ok_completed += good and not truncated
        if i < 8:
            samples.append({"prompt_ix": i % len(PROMPTS), "out": snippet, "parses": good})
        if not good:
            fails.append({"i": i, "out": snippet, "stop_type": resp.get("stop_type"), "truncated": truncated})
        elif False:
            pass
        if (i + 1) % 25 == 0:
            print(f"  {i + 1}/{n}: {ok} parse-valid", file=sys.stderr)
    return {
        "n": n,
        "parse_ok": ok,
        "rate": ok / max(n, 1),
        "completed": completed,
        "parse_ok_completed": ok_completed,
        "cap_retries": retries,
        "samples": samples,
        "failures": fails,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model")
    ap.add_argument("--server-url")
    ap.add_argument("--n", type=int, default=200)
    ap.add_argument("--n-free", type=int, default=100)
    ap.add_argument("--port", type=int, default=8123)
    args = ap.parse_args()

    server = None
    url = args.server_url
    if not url:
        if not args.model:
            sys.exit("need --model <gguf> or --server-url")
        url = f"http://127.0.0.1:{args.port}"
        server = subprocess.Popen(
            ["llama-server", "-m", args.model, "--port", str(args.port), "-ngl", "99", "-c", "2048"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        print("waiting for llama-server ...", file=sys.stderr)
        for _ in range(120):
            try:
                urllib.request.urlopen(url + "/health", timeout=2)
                break
            except Exception:
                time.sleep(1)
        else:
            sys.exit("llama-server did not become healthy")

    try:
        grammar = runtime_grammar()
        print(f"constrained arm (n={args.n}) ...", file=sys.stderr)
        constrained = arm(url, args.n, grammar)
        print(f"unconstrained arm (n={args.n_free}) ...", file=sys.stderr)
        unconstrained = arm(url, args.n_free, None)
    finally:
        if server:
            server.terminate()
            server.wait()

    results = {
        "date": time.strftime("%Y-%m-%d"),
        "constrained": constrained,
        "unconstrained": unconstrained,
    }
    (HERE / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps({k: v for k, v in results.items() if k != "date"}, default=str)[:400])
    print(
        f"\nconstrained: {constrained['parse_ok']}/{constrained['n']} parse-valid"
        f"\nunconstrained: {unconstrained['parse_ok']}/{unconstrained['n']} parse-valid"
    )
    return 0 if constrained["parse_ok"] == constrained["n"] else 1


if __name__ == "__main__":
    sys.exit(main())
