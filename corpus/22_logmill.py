import json
import sys


def g(x):
    return f"{x:g}"


def avg(xs):
    return sum(xs) / len(xs)


def classify(lvl, ms, why):
    if lvl == "ERROR":
        return why
    try:
        return int(ms)
    except ValueError:
        try:
            return float(ms)
        except ValueError:
            return None


def fetch(p):
    with open(p) as fh:
        return fh.read().splitlines()


def med(xs):
    s = sorted(xs)
    return s[len(s) // 2]


def bar(n):
    s = ""
    while len(s) < n:
        s += "#"
    return s


def counts(xs):
    m = {}
    for x in xs:
        m[x] = m.get(x, 0) + 1
    return m


def stat(k, v, sc):
    ms = [e["ms"] for e in v]
    print(f"{k}: reqs {len(v)} avg {g(avg(ms))} med {g(med(ms))} score {sc}")


def faults(fails):
    parts = [f"{k} {v}" for k, v in counts([s for s, _ in fails]).items()]
    print(f"errors by svc: {', '.join(parts)}")


def peak(mins):
    best = max(counts(mins).items(), key=lambda p: p[1])
    print(f"peak minute: {best[0]} x{best[1]}")


def lag(entries, slow):
    rows = [f"{e['svc']} {g(e['ms'])}" for e in entries if e["ms"] >= slow]
    msg = "none" if not rows else ", ".join(rows)
    print(f"slow >={slow}ms: {msg}")


def rank(groups, shown):
    tops = sorted(groups.items(), key=lambda kv: len(kv[1]), reverse=True)[:shown]
    print(f"top {shown} by traffic: {', '.join(f'{k} {len(v)}' for k, v in tops)}")


def worst(scored):
    name, val = max(scored, key=lambda t: t[1])
    print(f"worst: {name} score {val}")


def hist(groups):
    for k, v in groups.items():
        print(f"{k} {bar(len(v))}")


def footer(files, lines, bad):
    print(f"scanned {files} files, {lines} lines, {bad} bad")


base = '{"title": "logmill", "sources": ["west.log"]}'
path = sys.argv[1] if len(sys.argv) > 1 else "logmill.json"
try:
    spec = open(path).read()
except OSError:
    spec = base
try:
    job = json.loads(spec)
except ValueError:
    print("note: bad job spec, using defaults")
    job = json.loads(base)
title = job.get("title", "logmill")
slow = job.get("slow_ms", 150)
shown = job.get("top", 1)
src = job.get("sources", [])

sev = {
    "ERROR": 3,
    "WARN": 1,
}

entries = []
fails = []
tags = []
mins = []
bad = 0
missing = 0
seen = 0

for p in src:
    try:
        ls = fetch(p)
    except OSError:
        missing += 1
        continue
    for ln in ls:
        seen += 1
        f = ln.split(",")
        if len(f) == 5:
            mins.append(f[0][0:5])
            v = classify(f[1], f[3], f[4])
            if v is None:
                bad += 1
            elif isinstance(v, str):
                fails.append((f[2], v))
                tags.append((f[2], f[1]))
            else:
                entries.append({"svc": f[2], "ms": v})
                tags.append((f[2], f[1]))
        else:
            bad += 1

print(f"== {title} ==")
print(f"files {len(src) - missing} missing {missing}")
print(f"lines {seen} reqs {len(entries)} errs {len(fails)} bad {bad}")

groups = {}
for e in entries:
    groups.setdefault(e["svc"], []).append(e)
scored = []
for k, v in groups.items():
    sc = sum(sev.get(l, 0) for s, l in tags if s == k)
    scored.append((k, sc))
    stat(k, v, sc)

faults(fails)
peak(mins)
lag(entries, slow)
rank(groups, shown)
worst(scored)
hist(groups)

footer(len(src) - missing, seen, bad)
