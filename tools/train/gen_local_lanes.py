#!/usr/bin/env python3
"""Generate bench/dbench answer lanes from a LOCAL mlx model (curt-coder).

Replicates the frozen single-shot protocol of tools/bench/gen_lanes.py —
same prompts, temp 1.0, 3 samples, same code-block extraction, same
answers/ layout — so grade_bench.py / grade_dbench.py work unchanged.

Arms (the finetune-probe comparison):
  base-sheet   base model, CHEATSHEET.md system prompt (in-context arm)
  base-short   base model, short system prompt (floor: no curt knowledge)
  ft-short     base + curt-coder LoRA adapter, short system prompt
               (the weights arm — curt from training, not context)

Usage:
  .train-venv/bin/python tools/train/gen_local_lanes.py <tag> [arms...]
  # writes tools/{bench,dbench}/answers/curt_<arm>_<tag>/s{1..3}/
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "bench"))
from gen_lanes import parse_prompts  # noqa: E402

MODEL = "mlx-community/Qwen2.5-Coder-1.5B-Instruct-4bit"
ADAPTER = ROOT / "data" / "curtcoder" / "adapters-1.5b"
SHEET = (ROOT / "CHEATSHEET.md").read_text()
SHORT = "You write curt programs. Reply with ONLY a curt code block — no prose."
SAMPLES = 3
MAX_TOKENS = 900

ARMS = {
    "base-sheet": (None, SHEET + "\n\n" + SHORT),
    "base-short": (None, SHORT),
    "ft-short": (str(ADAPTER), SHORT),
}


def extract(text: str) -> str:
    m = re.search(r"```(?:curt)?\n(.*?)```", text, re.DOTALL)
    return (m.group(1) if m else text).strip() + "\n"


def main() -> int:
    from mlx_lm import generate, load
    from mlx_lm.sample_utils import make_sampler

    tag = sys.argv[1]
    arms = sys.argv[2:] or list(ARMS)
    suites = [
        (ROOT / "tools/bench", parse_prompts(ROOT / "tools/bench/PROMPTS.md", lang_sub=True)),
        (ROOT / "tools/dbench", parse_prompts(ROOT / "tools/dbench/PROMPTS.md")),
    ]
    for arm in arms:
        adapter, system = ARMS[arm]
        model, tokenizer = load(MODEL, adapter_path=adapter)
        n = 0
        for suite_dir, prompts in suites:
            for s in range(1, SAMPLES + 1):
                for task, prompt in prompts.items():
                    dest = (suite_dir / "answers" / f"curt_{arm}_{tag}"
                            / f"s{s}" / f"{task}.curt")
                    if dest.exists():
                        continue
                    msgs = [{"role": "system", "content": system},
                            {"role": "user", "content": prompt}]
                    text = generate(
                        model, tokenizer,
                        prompt=tokenizer.apply_chat_template(
                            msgs, add_generation_prompt=True),
                        max_tokens=MAX_TOKENS,
                        sampler=make_sampler(temp=1.0),
                    )
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    dest.write_text(extract(text))
                    n += 1
                    print(f"{arm} s{s} {task}", flush=True)
        del model
        print(f"{arm}: {n} generations", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
