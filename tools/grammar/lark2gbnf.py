#!/usr/bin/env python3
"""Generate curt.gbnf (llama.cpp) from curt.lark — mechanical and deterministic.

Rules convert token-by-token (the .lark file is written in a restricted
dialect: literals, terminal refs, rule refs, parens, ?, *, +, |). Terminals
convert via TERMINAL_MAP, which pins the EXACT lark regex source — if the
.lark terminal changes, this script fails loudly instead of drifting.

NAME excludes keywords EXACTLY via a generated prefix-trie complement (GBNF
has no lookahead, so the exclusion is encoded structurally). History: the
first version widened NAME to include keywords; gd-b-oss MEASURED that
widening leaking 30% keyword-shaped Python drift through the mask — the
trie complement closed it.

Usage: tools/grammar/lark2gbnf.py   (writes tools/grammar/curt.gbnf)
"""

import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
SRC = HERE / "curt.lark"
OUT = HERE / "curt.gbnf"

# (exact lark regex source) -> gbnf production body
TERMINAL_MAP = {
    "NUMBER": (
        r"/[0-9]+(\.[0-9]+)?([ui](8|16|32|64))?/",
        '[0-9]+ ("." [0-9]+)? ([ui] ("8" | "16" | "32" | "64"))?',
    ),
    "INT": (r"/[0-9]+/", "[0-9]+"),
    "STRING": (
        r'''/"(\\.|[^"\\])*"|'[^'\n]*'/''',
        '"\\"" ("\\\\" [^\\n] | [^\\"\\\\\\n])* "\\"" | "\'" [^\'\\n]* "\'"',
    ),
    "TNAME": (r"/[A-Z][A-Za-z0-9_]*/", "[A-Z] [A-Za-z0-9_]*"),
    "NAME": (
        r"/(?!(?:if|else|while|for|in|match|type|ret|go|pub|and|or|not)(?![A-Za-z0-9_]))[a-z_][A-Za-z0-9_]*/",
        # EXACT keyword exclusion, generated as a prefix-trie complement
        # (GBNF has no lookahead; gd-b-oss MEASURED the widened form leaking
        # 30% keyword-shaped Python drift through the mask)
        None,  # filled by name_rule() below
    ),
    "WS": (r"/[ \t]+/", "[ \\t]+"),
    "NL": (
        r"/[ \t]*(#[^\n]*)?\r?\n([ \t]*(#[^\n]*)?\r?\n)*[ \t]*/",
        '[ \\t]* ("#" [^\\n]*)? "\\r"? "\\n" ([ \\t]* ("#" [^\\n]*)? "\\r"? "\\n")* [ \\t]*',
    ),
}

KEYWORDS = ["if", "else", "while", "for", "in", "match", "type", "ret", "go", "pub", "and", "or", "not"]
NAME_CHAR = "[A-Za-z0-9_]"


def name_rule() -> str:
    """[a-z_][A-Za-z0-9_]* minus KEYWORDS, as a pure regular production.

    Trie walk: at a node reached by spelling a strict prefix of >=1 keyword,
    we may (a) stop (prefixes are legal names), (b) continue with a char that
    diverges from every keyword (then free tail), or (c) follow a keyword
    char deeper. At a node whose spelling IS a keyword, stopping is illegal:
    a continuation char is required.
    """

    def node(suffixes: list) -> str:
        # suffixes: remaining keyword tails reachable from this node
        is_kw_here = "" in suffixes
        heads = sorted({s[0] for s in suffixes if s})
        alts = []
        if heads:
            # a continuation char that matches no keyword tail -> free tail
            neg = "".join(heads)
            alts.append(f'[^{neg}] {NAME_CHAR}*' if False else f'{cls_minus(heads)} {NAME_CHAR}*')
            for h in heads:
                deeper = [s[1:] for s in suffixes if s and s[0] == h]
                alts.append(f'"{h}" {node(deeper)}')
        else:
            alts.append(f"{NAME_CHAR}+")
        body = " | ".join(alts)
        if is_kw_here:
            return f"({body})"  # must continue: the bare spelling is a keyword
        return f"({body})?"

    def cls_minus(heads: list) -> str:
        # name-continuation chars excluding the given lowercase heads
        ranges = []
        lo = "a"
        for h in heads + ["{"]:  # '{' = one past 'z'
            if lo < h:
                ranges.append(lo if chr(ord(lo)) == chr(ord(h) - 1) and False else f"{lo}-{chr(ord(h) - 1)}" if lo != chr(ord(h) - 1) else lo)
            lo = chr(ord(h) + 1)
        base = "".join(r for r in ranges)
        return f"[{base}A-Z0-9_]"

    by_first = sorted({k[0] for k in KEYWORDS})
    alts = [f"{cls_minus_first(by_first)} {NAME_CHAR}*"]
    for c in by_first:
        tails = [k[1:] for k in KEYWORDS if k[0] == c]
        alts.append(f'"{c}" {node(tails)}')
    return " | ".join(alts)


def cls_minus_first(heads: list) -> str:
    # legal first chars [a-z_] excluding keyword first-letters
    ranges = []
    lo = "a"
    for h in heads + ["{"]:
        if lo < h:
            ranges.append(f"{lo}-{chr(ord(h) - 1)}" if lo != chr(ord(h) - 1) else lo)
        lo = chr(ord(h) + 1)
    return f"[{''.join(ranges)}_]"


TOKEN_RE = re.compile(
    r'\s*(?:(?P<lit>"(?:\\.|[^"\\])*")|(?P<term>[A-Z][A-Z0-9_]*)'
    r"|(?P<rule>_?[a-z][a-z0-9_]*)|(?P<sym>[()|?*+]))"
)


def gbnf_name(lark_name: str) -> str:
    return lark_name.lstrip("_").replace("_", "-")


def convert_rhs(rhs: str, rules: set, terminals: set, line: str) -> str:
    out, pos = [], 0
    while pos < len(rhs):
        m = TOKEN_RE.match(rhs, pos)
        if not m:
            if rhs[pos:].strip():
                sys.exit(f"unconvertible token at: {rhs[pos:]!r} in {line!r}")
            break
        pos = m.end()
        if m.group("lit"):
            out.append(m.group("lit"))
        elif m.group("term"):
            if m.group("term") not in terminals:
                sys.exit(f"unknown terminal {m.group('term')} in {line!r}")
            out.append(gbnf_name(m.group("term").lower()))
        elif m.group("rule"):
            if m.group("rule") not in rules:
                sys.exit(f"unknown rule {m.group('rule')} in {line!r}")
            out.append(gbnf_name(m.group("rule")))
        else:
            sym = m.group("sym")
            if sym in "?*+" and out:
                out[-1] = out[-1] + sym
            else:
                out.append(sym)
    # re-join, keeping ?,*,+ glued and |,(,) spaced
    text = " ".join(out)
    return re.sub(r"\(\s+", "(", re.sub(r"\s+\)", ")", text))


def main() -> None:
    rule_lines, term_lines = [], []
    for raw in SRC.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("//"):
            continue
        name = line.split(":", 1)[0].strip()
        if re.fullmatch(r"[A-Z][A-Z0-9_]*", name):
            term_lines.append(line)
        elif re.fullmatch(r"_?[a-z][a-z0-9_]*", name):
            rule_lines.append(line)
        else:
            sys.exit(f"unparsable line: {line!r}")

    rules = {l.split(":", 1)[0].strip() for l in rule_lines}
    terminals = {l.split(":", 1)[0].strip() for l in term_lines}

    # whitespace plumbing: the lark side spells these as rule->terminal
    # aliases (_ws: WS?, ws1: WS, _nl: NL); flattened here so the sanitized
    # rule name and the terminal name don't collide (both would be "ws").
    ALIASES = {
        "_ws": "[ \\t]*",
        "ws1": "[ \\t]+",
        "_nl": TERMINAL_MAP["NL"][1],
    }
    ALIAS_TERMINALS = {"WS", "NL"}

    out = [
        "# curt v0.1 GBNF (llama.cpp) — GENERATED by lark2gbnf.py from curt.lark.",
        "# Do not hand-edit. NAME excludes keywords exactly (prefix-trie",
        "# complement — no lookahead needed). The Rust parser is the oracle.",
        "",
        "root ::= start",
    ]
    for line in rule_lines:
        name, rhs = (part.strip() for part in line.split(":", 1))
        if name in ALIASES:
            out.append(f"{gbnf_name(name)} ::= {ALIASES[name]}")
            continue
        out.append(f"{gbnf_name(name)} ::= {convert_rhs(rhs, rules, terminals, line)}")
    for line in term_lines:
        name, rhs = (part.strip() for part in line.split(":", 1))
        want_src, body = TERMINAL_MAP[name]
        if rhs != want_src:
            sys.exit(
                f"terminal {name} changed in curt.lark ({rhs!r}) but TERMINAL_MAP "
                f"still pins {want_src!r} — update the map deliberately"
            )
        if name in ALIAS_TERMINALS:
            continue  # flattened into the alias rules above
        if name == "NAME":
            body = name_rule()
        out.append(f"{gbnf_name(name.lower())} ::= {body}")
    OUT.write_text("\n".join(out) + "\n")
    print(f"wrote {OUT.name}: {len(rule_lines)} rules + {len(term_lines)} terminals")


if __name__ == "__main__":
    main()
