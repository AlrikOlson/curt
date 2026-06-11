#!/usr/bin/env python3
"""Deterministic seed generator — template families x parameter grids.

Each seed is (id, family, instruction, python_source). The generator
defines the transpiler's required subset; everything downstream is
execution-verified. Fully deterministic: a fixed PRNG seeded per family
draws value lists, so re-runs are byte-identical.
"""

import itertools
import random

NAME_POOLS = {
    "nums": ["nums", "vals", "xs", "data", "scores"],
    "words": ["words", "names", "items", "parts", "tokens"],
    "acc": ["total", "acc", "result", "out"],
}
SENTENCES = [
    "the quick brown fox jumps over the lazy dog",
    "pack my box with five dozen liquor jugs",
    "a stitch in time saves nine every single day",
    "all that glitters is not gold said the wise old owl",
    "to be or not to be that is the question",
    "an apple a day keeps the doctor away they say",
]
SEPS = [" ", ",", "-", ":"]


def int_lists(rng, n):
    return [[rng.randint(-9, 99) for _ in range(rng.randint(5, 9))] for _ in range(n)]


def gen():
    seeds = []

    def emit(family, instruction, src, **params):
        seeds.append({
            "id": f"{family}_{len(seeds):05d}",
            "family": family,
            "instruction": instruction,
            "python": src,
            "params": params,
        })

    # F1 filter_sum — sum of transformed elements passing a predicate
    rng = random.Random(101)
    for vals in int_lists(rng, 300):
        for thr, mul in itertools.product((0, 2, 5, 10), (1, 2, 3)):
            v = NAME_POOLS["nums"][thr % 5]
            body = "x" if mul == 1 else f"x * {mul}"
            emit(
                "filter_sum",
                f"Given the list {vals}, print the sum of {'each element' if mul == 1 else f'{mul} times each element'} strictly greater than {thr}.",
                f"{v} = {vals}\nprint(sum({body} for x in {v} if x > {thr}))\n",
                vals=vals, thr=thr, mul=mul,
            )

    # F2 count_condition — count elements matching a predicate
    rng = random.Random(102)
    for vals in int_lists(rng, 330):
        for op, thr in itertools.product(("<", ">", "=="), (0, 3, 7)):
            emit(
                "count_condition",
                f"Given the list {vals}, print how many elements are {op} {thr}.",
                f"xs = {vals}\nprint(len([x for x in xs if x {op} {thr}]))\n",
                vals=vals, op=op, thr=thr,
            )

    # F3 extremes — min/max/sum/sorted-first of a list
    rng = random.Random(103)
    for vals in int_lists(rng, 300):
        for fn in ("min", "max", "sum"):
            emit(
                "extremes",
                f"Print the {fn} of the list {vals}.",
                f"xs = {vals}\nprint({fn}(xs))\n",
                vals=vals, fn=fn,
            )
        emit(
            "extremes",
            f"Print the smallest two elements of {vals} on one line separated by a space.",
            f"xs = {vals}\ns = sorted(xs)\nprint(f\"{{s[0]}} {{s[1]}}\")\n",
            vals=vals, fn="sorted2",
        )

    # F4 word_transform — filter+case-transform words, join
    rng = random.Random(104)
    for sent, minlen, case, sep in itertools.product(SENTENCES, (2, 3, 4, 5), ("upper", "lower"), SEPS):
        emit(
            "word_transform",
            f'Split "{sent}" on spaces, keep words longer than {minlen} characters, convert them to {case}case, and print them joined by "{sep}".',
            f's = "{sent}"\nprint("{sep}".join(w.{case}() for w in s.split(" ") if len(w) > {minlen}))\n',
            sent=sent, minlen=minlen, case=case, sep=sep,
        )

    # F5 running_total — while-loop accumulation
    for start, limit, step in itertools.product((1, 2, 3), (20, 35, 50, 75), (3, 4, 7)):
        emit(
            "running_total",
            f"Starting from {start}, repeatedly add {step} while the running value stays below {limit}; print how many additions occurred and the final value, space-separated.",
            f"v = {start}\nn = 0\nwhile v < {limit}:\n    v += {step}\n    n += 1\nprint(f\"{{n}} {{v}}\")\n",
            start=start, limit=limit, step=step,
        )

    # F6 classify — if/elif/else banding
    rng = random.Random(106)
    for vals in int_lists(rng, 280):
        for lo, hi in ((0, 10), (5, 20), (10, 50)):
            emit(
                "classify",
                f"For each value in {vals}, print one line: \"low\" if it is below {lo}, \"high\" if it is above {hi}, else \"mid\".",
                f"xs = {vals}\nfor x in xs:\n    if x < {lo}:\n        print(\"low\")\n    elif x > {hi}:\n        print(\"high\")\n    else:\n        print(\"mid\")\n",
                vals=vals, lo=lo, hi=hi,
            )

    # F7 dedup_order — first-occurrence dedup
    rng = random.Random(107)
    for _ in range(420):
        vals = [rng.randint(0, 6) for _ in range(rng.randint(6, 10))]
        emit(
            "dedup_order",
            f"Remove duplicates from {vals} keeping first occurrences in order; print the result joined by spaces.",
            f"xs = {vals}\nseen = []\nfor x in xs:\n    if x not in seen:\n        seen.append(x)\nprint(\" \".join(str(x) for x in seen))\n",
            vals=vals,
        )

    # F8 validate — count valid/invalid by range, sum valid
    rng = random.Random(108)
    for _ in range(400):
        vals = [rng.randint(-20, 150) for _ in range(rng.randint(5, 8))]
        lo, hi = rng.choice([(0, 100), (0, 120), (10, 99)])
        emit(
            "validate",
            f"For the values {vals}, a value is valid when {lo} <= value <= {hi}. Print three lines: the valid count, the invalid count, and the sum of valid values.",
            f"xs = {vals}\nok = [x for x in xs if {lo} <= x <= {hi}]\nprint(len(ok))\nprint(len(xs) - len(ok))\nprint(sum(ok))\n",
            vals=vals, lo=lo, hi=hi,
        )

    # F9 fn_def — define and apply a small function
    for a, b in itertools.product((2, 3, 5, 7), (1, 4, 6, 9)):
        for shape in ("affine", "clamp"):
            if shape == "affine":
                emit(
                    "fn_def",
                    f"Define a function f(x) = {a}*x + {b} and print f(3), f(10), and f(-2) on separate lines.",
                    f"def f(x): return {a} * x + {b}\nprint(f(3))\nprint(f(10))\nprint(f(-2))\n",
                    a=a, b=b, shape=shape,
                )
            else:
                emit(
                    "fn_def",
                    f"Define a function clamp(x) returning {a} when x is below {a}, {a + b + 10} when x is above {a + b + 10}, else x; print clamp(0), clamp({a + 2}), clamp(99) on separate lines.",
                    f"def clamp(x):\n    if x < {a}:\n        return {a}\n    if x > {a + b + 10}:\n        return {a + b + 10}\n    return x\nprint(clamp(0))\nprint(clamp({a + 2}))\nprint(clamp(99))\n",
                    a=a, b=b, shape=shape,
                )

    # F10 char_stats — character-level counting over a string
    for sent, ch in itertools.product(SENTENCES, "aeiout"):
        emit(
            "char_stats",
            f'Count occurrences of the character "{ch}" in "{sent}" and print the count.',
            f's = "{sent}"\nprint(len([c for c in s if c == "{ch}"]))\n',
            sent=sent, ch=ch,
        )

    # F11 average — integer average with guard
    rng = random.Random(111)
    for vals in int_lists(rng, 420):
        emit(
            "average",
            f"Print the integer average (floor division) of the positive elements of {vals}; print 0 if there are none.",
            f"xs = {vals}\npos = [x for x in xs if x > 0]\nif len(pos) > 0:\n    print(sum(pos) // len(pos))\nelse:\n    print(0)\n",
            vals=vals,
        )

    # F12 slice_ops — slicing and indexing
    rng = random.Random(112)
    for vals in int_lists(rng, 380):
        k = rng.randint(1, 3)
        emit(
            "slice_ops",
            f"For the list {vals}, print the first {k} elements joined by commas, then the last element.",
            f"xs = {vals}\nprint(\",\".join(str(x) for x in xs[:{k}]))\nprint(xs[-1])\n" .replace("xs[-1]", f"xs[len(xs) - 1]"),
            vals=vals, k=k,
        )

    # F13 fizz_lines — modulo branching over a range
    for n, d1, d2 in itertools.product((10, 15, 20), (2, 3), (5, 7)):
        if d1 == d2:
            continue
        emit(
            "fizz_lines",
            f"For each i from 1 to {n} inclusive, print \"both\" if i is divisible by {d1} and {d2}, \"a\" if only by {d1}, \"b\" if only by {d2}, else i itself.",
            f"for i in range(1, {n + 1}):\n    if i % {d1} == 0 and i % {d2} == 0:\n        print(\"both\")\n    elif i % {d1} == 0:\n        print(\"a\")\n    elif i % {d2} == 0:\n        print(\"b\")\n    else:\n        print(i)\n",
            n=n, d1=d1, d2=d2,
        )

    return seeds


if __name__ == "__main__":
    s = gen()
    fams = {}
    for x in s:
        fams[x["family"]] = fams.get(x["family"], 0) + 1
    print(f"{len(s)} seeds across {len(fams)} families")
    for f, c in sorted(fams.items()):
        print(f"  {f:<16} {c}")
