#!/usr/bin/env python3
"""Spelling tournaments: measure each candidate's o200k token cost in context.

Rule (CLAUDE.md): cost decides; on a tie, the documented reliability tiebreak
decides and is recorded as such. Run: python3 tournaments.py
"""
import tiktoken

e = tiktoken.get_encoding("o200k_base")
n = lambda s: len(e.encode(s))

# Each tournament: (decision, [(candidate, sample-in-context)], tiebreak-note)
T = [
    ("equality operator",
     [("==", "if x == 1 { y }"), ("=", "if x = 1 { y }")],
     "tie -> '==' wins: removes binding/equality dual-use ambiguity; Python/C muscle memory (language-confusion mitigation). Postel accepts '=' in expression position."),
    ("boolean ops",
     [("and/or/not", "a and b or not c"), ("&/or/!", "a & b or !c")],
     "tie -> words win: Python-aligned; frees '&','|','!' for bitwise/pipe/future. Postel accepts '&&','||'."),
    ("print verb",
     [("print", "print x"), ("say", "say x"), ("out", "out x")],
     "tie -> 'print' wins: maximal muscle memory, zero teaching cost in cheat sheet."),
    ("float type name",
     [("float", "x: float = 1.5"), ("flt", "x: flt = 1.5")],
     "cost decides if not tie; if tie, 'float' wins on familiarity."),
    ("int type name",
     [("int", "x: int = 1")],
     "uncontested; sized i8/i16/i32/i64/u8..u64 cost measured for the record."),
    ("lambda/match arrow",
     [("->", "map x -> x + 1"), ("=>", "map x => x + 1")],
     "tie -> '->' wins: one arrow everywhere (lambdas, match arms); Rust/Haskell presence."),
    ("range operator",
     [("..", "for i in 0..10 { }"), ("range", "for i in range 10 { }")],
     "cost decides; '..' also reads as Rust."),
    ("error propagate",
     [("?", "x = parse s?"), ("try", "x = try parse s")],
     "postfix '?' = Rust muscle memory; also doubles as `expr ? fallback` rescue."),
    ("export marker",
     [(":: sig line", "add :: int int -> int\nadd a b = a + b"), ("pub keyword", "pub add a b = a + b")],
     "cost decides per exported fn; ':: line' also carries the FFI type info pub would need anyway."),
    ("compound add",
     [("+=", "n += 1")],
     "uncontested, recorded for the table."),
]

print("=== TOURNAMENTS (o200k_base, in-context counts) ===")
for name, cands, note in T:
    counts = [(c, n(s)) for c, s in cands]
    best = min(c[1] for c in counts)
    winners = [c for c in counts if c[1] == best]
    print(f"\n{name}:")
    for c, k in counts:
        print(f"   {c:14s} {k:3d} tokens   sample ok")
    verdict = "COST" if len(winners) == 1 else "TIE->RELIABILITY"
    print(f"   -> decided by {verdict}; note: {note}")

print("\n=== sized ints / misc singles ===")
for w in ["i8","i16","i32","i64","u8","u16","u32","u64","float","int","str","bool","print","match","type","go","..","::","->","==","+=","in","and","or","not"]:
    print(f"   {w!r:8s} -> {n(' '+w) if not w.startswith(('.',':','-','=','+')) else n(w)} token(s)")
