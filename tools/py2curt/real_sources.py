#!/usr/bin/env python3
"""Adapter: real function+assert problems -> stdout-graded pipeline seeds.

Each MBPP / HumanEval problem ships a solution and `assert f(args) == x`
tests. The pipeline grades stdout, so the adapter rewrites every problem
as a self-contained top-level program: the (normalized) solution followed
by one print per usable assert. The Python oracle then defines the
expected output exactly as for generated seeds.

Mechanical normalizations (semantics-free, applied to the Python source):
  - drop `typing` imports, type annotations, and docstrings
  - scalar results print directly; sequence results print space-joined
    (`print(" ".join(str(x) for x in r))`) so both languages render
    identically under the grader's numeric/bool token folding

Assert shapes used: `assert call(...) == literal` and bare
`assert call(...)` (truthiness). Anything else (isclose, set equality,
exception tests) is skipped; a problem with zero usable asserts is
rejected with tag `no-printable-tests`.
"""

import ast
import gzip
import json
import pathlib
import warnings

EXT = pathlib.Path(__file__).resolve().parents[2] / "data" / "external"

SCALAR = (int, float, str, bool)


class Normalize(ast.NodeTransformer):
    """Strip annotations, typing imports, and docstrings."""

    def visit_Import(self, node):
        node.names = [a for a in node.names if a.name != "typing"]
        return node if node.names else None

    def visit_ImportFrom(self, node):
        return None if node.module == "typing" else node

    def visit_AnnAssign(self, node):
        if node.value is None:
            return None
        return ast.copy_location(ast.Assign(targets=[node.target], value=node.value), node)

    def visit_FunctionDef(self, node):
        self.generic_visit(node)
        node.returns = None
        for a in node.args.args + node.args.posonlyargs + node.args.kwonlyargs:
            a.annotation = None
        if (
            node.body
            and isinstance(node.body[0], ast.Expr)
            and isinstance(node.body[0].value, ast.Constant)
            and isinstance(node.body[0].value.value, str)
        ):
            node.body = node.body[1:] or [ast.Pass()]
        return node


class RenameFns(ast.NodeTransformer):
    """Lowercase function names — curt reserves Capitalized ids for types."""

    def __init__(self, mapping):
        self.mapping = mapping

    def visit_FunctionDef(self, node):
        self.generic_visit(node)
        node.name = self.mapping.get(node.name, node.name)
        return node

    def visit_Name(self, node):
        node.id = self.mapping.get(node.id, node.id)
        return node


def fn_renames(src):
    return {
        n.name: n.name.lower()
        for n in ast.walk(ast.parse(src))
        if isinstance(n, ast.FunctionDef) and n.name != n.name.lower()
    }


def normalize(src, renames=None):
    tree = Normalize().visit(ast.parse(src))
    if renames:
        tree = RenameFns(renames).visit(tree)
    ast.fix_missing_locations(tree)
    return ast.unparse(tree)


def parse_asserts(test_src, rename=None):
    """Extract (call_source, is_truthy) from assert statements."""
    out = []
    tree = ast.parse(test_src)
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assert):
            continue
        t = node.test
        call = None
        truthy = False
        if (
            isinstance(t, ast.Compare)
            and len(t.ops) == 1
            and isinstance(t.ops[0], ast.Eq)
            and isinstance(t.left, ast.Call)
        ):
            call = t.left
        elif isinstance(t, ast.Call):
            call, truthy = t, True
        if call is None:
            continue
        if rename and isinstance(call.func, ast.Name) and call.func.id == "candidate":
            call.func.id = rename
        out.append((ast.unparse(call), truthy))
    return out


def probe(solution_src, call_srcs):
    """Run the solution once, eval each call, classify result shapes.

    Returns [(call_src, kind)] where kind is 'scalar' | 'seq', or raises.
    """
    env = {}
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", SyntaxWarning)
        exec(solution_src, env)  # noqa: S102 — benchmark solutions, oracle context
    shapes = []
    for c in call_srcs:
        r = eval(c, env)  # noqa: S307
        if isinstance(r, bool) or isinstance(r, SCALAR):
            shapes.append((c, "scalar"))
        elif isinstance(r, (list, tuple)) and all(isinstance(x, SCALAR) for x in r):
            shapes.append((c, "seq"))
        else:
            shapes.append((c, None))
    return shapes


def build_seed(sid, source, text, solution_src, asserts, extra):
    try:
        renames = fn_renames(solution_src)
        solution = normalize(solution_src, renames)
        if renames:
            asserts = [
                (normalize(c, renames).strip(), t) for c, t in asserts
            ]
    except SyntaxError:
        return {"id": sid, "status": "reject", "why": "py-syntax"}
    try:
        shapes = probe(solution, [c for c, _ in asserts])
    except Exception as e:  # noqa: BLE001 — any oracle failure is one taxonomy tag
        return {"id": sid, "status": "reject", "why": f"oracle-error:{type(e).__name__}"}
    usable = [(c, k) for c, k in shapes if k]
    if not usable:
        return {"id": sid, "status": "reject", "why": "no-printable-tests"}

    lines = [solution.rstrip()]
    for c, kind in usable:
        if kind == "scalar":
            lines.append(f"print({c})")
        else:
            lines.append(f"res = {c}")
            lines.append('print(" ".join(str(x) for x in res))')
    calls = "; ".join(c for c, _ in usable)
    note = " Sequence results print space-separated on one line." if any(
        k == "seq" for _, k in usable
    ) else ""
    instruction = (
        f"{text.strip()} Print the result of each of the following calls, "
        f"one per line: {calls}.{note}"
    )
    return {
        "id": sid,
        "family": source,
        "instruction": instruction,
        "python": "\n".join(lines) + "\n",
        "params": extra,
        "status": "seed",
    }


def he_doc(full_src, entry):
    """The docstring of the entry-point function, whitespace-collapsed."""
    try:
        fn = next(
            n for n in ast.walk(ast.parse(full_src))
            if isinstance(n, ast.FunctionDef) and n.name == entry
        )
        doc = ast.get_docstring(fn, clean=True)
        return " ".join(doc.split()) if doc else ""
    except (SyntaxError, StopIteration):
        return ""


def gen():
    """All adapted seeds + adapter-level rejects, deterministically ordered."""
    plus_ids = set()
    with (EXT / "mbppplus.jsonl").open() as f:
        for line in f:
            plus_ids.add(json.loads(line)["task_id"])

    items = []
    with (EXT / "mbpp.jsonl").open() as f:
        for line in f:
            row = json.loads(line)
            items.append(build_seed(
                f"mbpp_{row['task_id']:05d}",
                "mbpp",
                row["text"],
                row["code"],
                parse_asserts("\n".join(row["test_list"])),
                {"task_id": row["task_id"], "mbpp_plus": row["task_id"] in plus_ids,
                 "split": "train"},
            ))

    with gzip.open(EXT / "HumanEval.jsonl.gz", "rt") as f:
        for line in f:
            row = json.loads(line)
            num = int(row["task_id"].split("/")[1])
            items.append(build_seed(
                f"humaneval_{num:05d}",
                "humaneval",
                f"Write a function `{row['entry_point']}`. "
                + he_doc(row["prompt"] + row["canonical_solution"], row["entry_point"]),
                row["prompt"] + row["canonical_solution"],
                parse_asserts(row["test"], rename=row["entry_point"]),
                {"task_id": row["task_id"], "mbpp_plus": False, "split": "eval"},
            ))
    return items


if __name__ == "__main__":
    seeds = gen()
    ok = [s for s in seeds if s["status"] == "seed"]
    rej = [s for s in seeds if s["status"] != "seed"]
    print(f"{len(seeds)} problems -> {len(ok)} adapted seeds, {len(rej)} adapter rejects")
    tags = {}
    for r in rej:
        tags[r["why"]] = tags.get(r["why"], 0) + 1
    for why, n in sorted(tags.items(), key=lambda kv: -kv[1]):
        print(f"  {why:<28} {n}")
