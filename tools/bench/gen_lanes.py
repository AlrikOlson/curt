#!/usr/bin/env python3
"""Generate fresh single-shot curt answer lanes via the API.

Replicates the frozen-lane protocol exactly: the model receives the
current CHEATSHEET as its only language reference (system prompt) and
the task prompt verbatim (user message); one shot, no execution, no
retries; every output is frozen verbatim — including failures.

Usage:
  .ci-venv/bin/python tools/bench/gen_lanes.py v4
writes tools/{bench,dbench}/answers/curt_{haiku,sonnet}_<tag>/s{1..3}/.
"""

import json
import pathlib
import re
import sys
import urllib.request
from concurrent.futures import ThreadPoolExecutor

ROOT = pathlib.Path(__file__).resolve().parents[2]
SHEET = (ROOT / "CHEATSHEET.md").read_text()
MODELS = {"haiku": "claude-haiku-4-5-20251001", "sonnet": "claude-sonnet-4-6"}
SAMPLES = 3

SYSTEM = SHEET + (
    "\n\nYou write curt programs. Reply with ONLY a curt code block — no prose."
)


def api_key():
    return (ROOT / "data" / "external" / "anthropic.key").read_text().strip()


def parse_prompts(path, lang_sub=False):
    """task-id -> prompt text from a PROMPTS.md (## task sections)."""
    text = pathlib.Path(path).read_text()
    out = {}
    for m in re.finditer(r"^## (\S+)\n(.*?)(?=^## |\Z)", text, re.M | re.S):
        prompt = m.group(2).strip()
        if lang_sub:
            prompt = prompt.replace("<LANG>", "curt")
        out[m.group(1)] = prompt
    return out


def generate(model, prompt, key):
    body = {
        "model": model, "max_tokens": 900, "temperature": 1.0,
        "system": [{"type": "text", "text": SYSTEM,
                    "cache_control": {"type": "ephemeral"}}],
        "messages": [{"role": "user", "content": prompt}],
    }
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(body).encode(),
        headers={"x-api-key": key, "anthropic-version": "2023-06-01",
                 "content-type": "application/json"},
    )
    import time
    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                text = json.load(r)["content"][0]["text"]
            m = re.search(r"```(?:curt)?\n(.*?)```", text, re.DOTALL)
            return (m.group(1) if m else text).strip() + "\n"
        except Exception:  # noqa: BLE001 — retry transient API errors
            time.sleep(2 ** attempt + 1)
    raise RuntimeError(f"generation failed: {model}")


def main():
    tag = sys.argv[1]
    key = api_key()
    suites = [
        (ROOT / "tools/bench", parse_prompts(ROOT / "tools/bench/PROMPTS.md", lang_sub=True)),
        (ROOT / "tools/dbench", parse_prompts(ROOT / "tools/dbench/PROMPTS.md")),
    ]
    jobs = []
    for suite_dir, prompts in suites:
        for short, model in MODELS.items():
            for s in range(1, SAMPLES + 1):
                for task, prompt in prompts.items():
                    dest = suite_dir / "answers" / f"curt_{short}_{tag}" / f"s{s}" / f"{task}.curt"
                    jobs.append((model, prompt, dest))
    print(f"{len(jobs)} generations")

    def run(job):
        model, prompt, dest = job
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(generate(model, prompt, key))
        return dest

    with ThreadPoolExecutor(max_workers=8) as ex:
        done = list(ex.map(run, jobs))
    print(f"froze {len(done)} files")


if __name__ == "__main__":
    main()
