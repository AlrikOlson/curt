#!/usr/bin/env python3
"""py2curt — Python-subset → curt transpiler (semantics-aligned, not
line-by-line).

The supported subset is defined by gen_seeds.py plus the subset-clean
corpus twins: ints/floats/strings/lists, (aug-)assignment, for/while/
if-elif-else, simple function defs, print, f-strings, comprehensions,
and a mapped builtin/method vocabulary. Comprehensions become verb
pipelines (keep/map), never transliterated loops — the anti-negative-
transfer rule. Unsupported nodes raise Unsupported with a taxonomy tag.
"""

import ast


class Unsupported(Exception):
    def __init__(self, tag, detail=""):
        super().__init__(f"{tag}: {detail}")
        self.tag = tag


STR_VARS: set = set()
USER_FNS: set = set()


def transpile(src: str) -> str:
    tree = ast.parse(src)
    STR_VARS.clear()
    USER_FNS.clear()
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            USER_FNS.add(node.name)
    for node in ast.walk(tree):
        if (
            isinstance(node, ast.Assign)
            and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name)
            and (
                (isinstance(node.value, ast.Constant) and isinstance(node.value.value, str))
                or isinstance(node.value, ast.JoinedStr)
            )
        ):
            STR_VARS.add(node.targets[0].id)
    out = []
    for node in tree.body:
        out.append(stmt(node, 0))
    return "\n".join(out) + "\n"


IND = "  "

BIN = {ast.Add: "+", ast.Sub: "-", ast.Mult: "*", ast.Div: "/", ast.Mod: "%",
       ast.Pow: "**", ast.FloorDiv: "/", ast.BitXor: "^"}
CMP = {ast.Eq: "==", ast.NotEq: "!=", ast.Lt: "<", ast.LtE: "<=",
       ast.Gt: ">", ast.GtE: ">=", ast.In: "in"}


def stmt(n, d):
    pad = IND * d
    if isinstance(n, ast.Assign):
        if len(n.targets) != 1:
            raise Unsupported("multi-assign")
        t = n.targets[0]
        if isinstance(t, ast.Name):
            return f"{pad}{t.id} = {expr(n.value)}"
        if isinstance(t, ast.Subscript):
            return f"{pad}{expr(t.value)}[{expr(t.slice)}] = {expr(n.value)}"
        if isinstance(t, ast.Tuple) and all(isinstance(e, ast.Name) for e in t.elts):
            names = ", ".join(e.id for e in t.elts)
            return f"{pad}({names}) = {expr(n.value)}"
        raise Unsupported("assign-target", ast.dump(t)[:40])
    if isinstance(n, ast.AugAssign):
        op = BIN.get(type(n.op))
        if op is None:
            raise Unsupported("augassign-op")
        if isinstance(n.target, ast.Name):
            return f"{pad}{n.target.id} {op}= {expr(n.value)}"
        if isinstance(n.target, ast.Subscript) and not isinstance(n.target.slice, ast.Slice):
            t = expr(n.target)
            return f"{pad}{t} = {t} {op} {paren(n.value)}"
        raise Unsupported("augassign-target")
    if isinstance(n, ast.Expr):
        return f"{pad}{expr_stmt(n.value)}"
    if isinstance(n, ast.For):
        if not isinstance(n.target, ast.Name) or n.orelse:
            raise Unsupported("for-shape")
        body = "\n".join(stmt(s, d + 1) for s in n.body)
        return f"{pad}for {n.target.id} in {expr(n.iter)} {{\n{body}\n{pad}}}"
    if isinstance(n, ast.While):
        if n.orelse:
            raise Unsupported("while-else")
        body = "\n".join(stmt(s, d + 1) for s in n.body)
        return f"{pad}while {expr(n.test)} {{\n{body}\n{pad}}}"
    if isinstance(n, ast.If):
        body = "\n".join(stmt(s, d + 1) for s in n.body)
        s = f"{pad}if {expr(n.test)} {{\n{body}\n{pad}}}"
        if n.orelse:
            if len(n.orelse) == 1 and isinstance(n.orelse[0], ast.If):
                return s + " else " + stmt(n.orelse[0], d).lstrip()
            els = "\n".join(stmt(x, d + 1) for x in n.orelse)
            s += f" else {{\n{els}\n{pad}}}"
        return s
    if isinstance(n, ast.FunctionDef):
        if n.decorator_list or any(
            getattr(n.args, f) for f in ("kwonlyargs", "posonlyargs", "defaults", "vararg", "kwarg")
        ):
            raise Unsupported("def-shape", n.name)
        if not n.args.args:
            # curt application is juxtaposition; nullary defs have no
            # faithful equivalent (a binding evaluates once, eagerly)
            raise Unsupported("def-noargs", n.name)
        params = " ".join(a.arg for a in n.args.args)
        # single return-expression body collapses to an equation
        if len(n.body) == 1 and isinstance(n.body[0], ast.Return) and n.body[0].value is not None:
            return f"{pad}{n.name} {params} = {expr(n.body[0].value)}"
        body = "\n".join(stmt(s, d + 1) for s in n.body)
        return f"{pad}{n.name} {params} = {{\n{body}\n{pad}}}"
    if isinstance(n, ast.Return):
        if n.value is None:
            raise Unsupported("bare-return")
        return f"{pad}ret {expr(n.value)}"
    raise Unsupported("stmt", type(n).__name__)


def expr_stmt(n):
    # print(...) at statement position
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "print":
        if len(n.args) == 0:
            return 'print ""'
        if len(n.args) == 1:
            a = expr(n.args[0])
            return f"print ({a})" if needs_parens(n.args[0]) else f"print {a}"
        # multi-arg print space-joins via interpolation
        holes = " ".join("{" + hole(a) + "}" for a in n.args)
        return f'print "{holes}"'
    raise Unsupported("expr-stmt", type(n).__name__)


def hole(n):
    """Interpolation hole content — must not contain double quotes."""
    e = expr(n)
    if '"' in e:
        raise Unsupported("quote-in-hole", e[:30])
    return e


def needs_parens(n):
    return not isinstance(n, (ast.Name, ast.Constant, ast.Subscript))


def expr(n):
    if isinstance(n, ast.Constant):
        if isinstance(n.value, bool):
            return "true" if n.value else "false"
        if isinstance(n.value, (int, float)):
            return repr(n.value)
        if isinstance(n.value, str):
            body = n.value.replace("\\", "\\\\").replace('"', '\\"').replace("{", "\\{").replace("\n", "\\n")
            return f'"{body}"'
        raise Unsupported("const", repr(n.value)[:30])
    if isinstance(n, ast.Name):
        return n.id
    if isinstance(n, ast.List):
        return "[" + ", ".join(expr(e) for e in n.elts) + "]"
    if isinstance(n, ast.Dict):
        # string-keyed dicts map to v0.3 map literals; other keys unsupported
        if not n.keys or any(
            k is None or not (isinstance(k, ast.Constant) and isinstance(k.value, str))
            for k in n.keys
        ):
            if not n.keys:
                return "{}"
            raise Unsupported("dict-key")
        entries = ", ".join(f"{expr(k)}: {expr(v)}" for k, v in zip(n.keys, n.values))
        return "{" + entries + "}"
    if isinstance(n, ast.Tuple):
        return "(" + ", ".join(expr(e) for e in n.elts) + ")"
    if isinstance(n, ast.BinOp):
        op = BIN.get(type(n.op))
        if op is None:
            raise Unsupported("binop", type(n.op).__name__)
        return f"{paren(n.left)} {op} {paren(n.right)}"
    if isinstance(n, ast.UnaryOp):
        if isinstance(n.op, ast.USub):
            return f"0 - {paren(n.operand)}" if not isinstance(n.operand, ast.Constant) else f"-{expr(n.operand)}"
        if isinstance(n.op, ast.Not):
            return f"not {paren(n.operand)}"
        raise Unsupported("unary", type(n.op).__name__)
    if isinstance(n, ast.BoolOp):
        op = "and" if isinstance(n.op, ast.And) else "or"
        return f" {op} ".join(paren(v) for v in n.values)
    if isinstance(n, ast.Compare):
        if len(n.ops) == 1:
            if isinstance(n.ops[0], ast.NotIn):
                return f"not ({paren(n.left)} in {paren(n.comparators[0])})"
            op = CMP.get(type(n.ops[0]))
            if op is None:
                raise Unsupported("cmp", type(n.ops[0]).__name__)
            return f"{paren(n.left)} {op} {paren(n.comparators[0])}"
        if len(n.ops) == 2 and all(type(o) in (ast.LtE, ast.Lt) for o in n.ops):
            # a <= x <= b chains split into a conjunction
            a, b = CMP[type(n.ops[0])], CMP[type(n.ops[1])]
            mid = paren(n.comparators[0])
            return f"{paren(n.left)} {a} {mid} and {mid} {b} {paren(n.comparators[1])}"
        raise Unsupported("cmp-chain")
    if isinstance(n, ast.IfExp):
        return f"if {expr(n.test)} {{ {expr(n.body)} }} else {{ {expr(n.orelse)} }}"
    if isinstance(n, ast.Subscript):
        if isinstance(n.slice, ast.Slice):
            if n.slice.step is not None:
                # xs[::-1] is the Python reversal idiom -> .rev
                s = n.slice.step
                if (
                    isinstance(s, ast.UnaryOp) and isinstance(s.op, ast.USub)
                    and isinstance(s.operand, ast.Constant) and s.operand.value == 1
                    and n.slice.lower is None and n.slice.upper is None
                ):
                    return f"{paren(n.value)}.rev"
                raise Unsupported("slice-step")
            lo = expr(n.slice.lower) if n.slice.lower else ""
            hi = expr(n.slice.upper) if n.slice.upper else ""
            return f"{paren(n.value)}[{lo}:{hi}]"
        return f"{paren(n.value)}[{expr(n.slice)}]"
    if isinstance(n, ast.JoinedStr):
        parts = []
        for v in n.values:
            if isinstance(v, ast.Constant):
                parts.append(v.value.replace("\\", "\\\\").replace('"', '\\"').replace("{", "\\{"))
            elif isinstance(v, ast.FormattedValue):
                if v.format_spec or v.conversion != -1:
                    raise Unsupported("fstring-spec")
                parts.append("{" + hole(v.value) + "}")
        return '"' + "".join(parts) + '"'
    if isinstance(n, (ast.ListComp, ast.GeneratorExp)):
        return comprehension(n)
    if isinstance(n, ast.Call):
        return call(n)
    if isinstance(n, ast.Lambda):
        a = n.args
        if (
            len(a.args) != 1
            or a.kwonlyargs or a.posonlyargs or a.defaults or a.vararg or a.kwarg
        ):
            raise Unsupported("lambda-shape")
        return f"({a.args[0].arg} -> {expr(n.body)})"
    raise Unsupported("expr", type(n).__name__)


def paren(n):
    e = expr(n)
    if isinstance(n, (ast.Name, ast.Constant, ast.Subscript, ast.List, ast.Tuple)):
        return e
    return f"({e})"


def comprehension(n):
    """[f(x) for x in xs if c] -> xs | keep (x -> c) | map (x -> f)."""
    if len(n.generators) != 1:
        raise Unsupported("comp-multi")
    g = n.generators[0]
    if not isinstance(g.target, ast.Name) or g.is_async:
        raise Unsupported("comp-target")
    var = g.target.id
    src = expr(g.iter)
    # iterating a string pipes its characters
    if (isinstance(g.iter, ast.Name) and g.iter.id in STR_VARS) or (
        isinstance(g.iter, ast.Constant) and isinstance(g.iter.value, str)
    ):
        src = f"{src}.chars" if isinstance(g.iter, ast.Name) else f"({src}).chars"
    elif not isinstance(g.iter, (ast.Name, ast.List)):
        src = f"({src})"
    stages = [src]
    for cond in g.ifs:
        stages.append(f"keep ({var} -> {expr(cond)})")
    body = expr(n.elt)
    if body != var:
        stages.append(f"map ({var} -> {body})")
    if len(stages) == 1:
        return src
    return " | ".join(stages)


# builtin functions -> UFCS verbs (receiver-first)
FN1 = {"len": "len", "sum": "sum", "min": "min", "max": "max", "sorted": "sort",
       "str": "str", "int": "int", "float": "float", "abs": None, "reversed": "rev"}
# methods -> verbs with same argument shape
METH = {"upper": "upper", "lower": "lower", "strip": "trim", "split": "split",
        "replace": "replace", "append": None, "join": None, "isdigit": "digit"}


def call(n):
    if isinstance(n.func, ast.Name):
        f = n.func.id
        if f == "range":
            args = " ".join(paren(a) for a in n.args)
            return f"range {args}"
        if f == "print":
            raise Unsupported("print-as-expr")
        if f in FN1 and len(n.args) == 1:
            verb = FN1[f]
            if verb is None:
                raise Unsupported("builtin", f)
            inner = n.args[0]
            # sum/min/etc over a comprehension fuses into the pipeline
            if isinstance(inner, (ast.ListComp, ast.GeneratorExp)):
                return f"{comprehension(inner)} | {verb}"
            return f"{paren(inner)}.{verb}"
        # list(range(..)) / list(comprehension) are identities: range and
        # pipelines already yield lists in curt
        if f == "list" and len(n.args) == 1:
            inner = n.args[0]
            if isinstance(inner, ast.Call) and isinstance(inner.func, ast.Name) \
                    and inner.func.id == "range":
                return f"({expr(inner)})"
            if isinstance(inner, (ast.ListComp, ast.GeneratorExp)):
                return comprehension(inner)
            raise Unsupported("builtin", "list")
        # min(a, b) / max(a, b) variadic forms become list verbs
        if f in ("min", "max") and len(n.args) > 1:
            items = ", ".join(expr(a) for a in n.args)
            return f"[{items}].{f}"
        if f not in USER_FNS:
            raise Unsupported("builtin", f)
        # user-defined function call: juxtaposition
        args = " ".join(paren(a) for a in n.args)
        return f"{f} {args}" if args else f"{f}()"
    if isinstance(n.func, ast.Attribute):
        recv, meth = n.func.value, n.func.attr
        if meth == "append" and len(n.args) == 1:
            # statement-level idiom handled here as concat expression
            raise Unsupported("append-as-expr")
        if meth == "join" and len(n.args) == 1:
            sep = expr(recv)
            inner = n.args[0]
            if isinstance(inner, (ast.ListComp, ast.GeneratorExp)):
                return f"{comprehension(inner)} | join {sep}"
            return f"{paren(inner)} | join {sep}"
        if meth in METH and METH[meth]:
            verb = METH[meth]
            args = " ".join(paren(a) for a in n.args)
            return f"{paren(recv)}.{verb} {args}".rstrip()
        if meth == "get" and len(n.args) == 2:
            # d.get(k, default) -> d[k] ? default
            return f"{paren(recv)}[{expr(n.args[0])}] ? {paren(n.args[1])}"
        raise Unsupported("method", meth)
    raise Unsupported("call-shape")


# x.append(y) at statement level -> x += [y]
_orig_stmt = stmt


def stmt(n, d):  # noqa: F811 — wrap to catch append statements
    if (
        isinstance(n, ast.Expr)
        and isinstance(n.value, ast.Call)
        and isinstance(n.value.func, ast.Attribute)
        and n.value.func.attr == "append"
        and isinstance(n.value.func.value, ast.Name)
        and len(n.value.args) == 1
    ):
        pad = IND * d
        return f"{pad}{n.value.func.value.id} += [{expr(n.value.args[0])}]"
    return _orig_stmt(n, d)


if __name__ == "__main__":
    import sys

    print(transpile(open(sys.argv[1]).read()), end="")
