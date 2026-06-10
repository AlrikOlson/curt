import sys

def lex(s):
    s = s.replace("(", " ( ").replace(")", " ) ")
    return [float(w) if w[0].isdigit() else w for w in s.split()]

def expr(ts):
    v, r = term(ts)
    while r and r[0] in "+-":
        v2, r2 = term(r[1:])
        v = v + v2 if r[0] == "+" else v - v2
        r = r2
    return v, r

def term(ts):
    v, r = factor(ts)
    while r and r[0] in "*/":
        v2, r2 = factor(r[1:])
        v = v * v2 if r[0] == "*" else v / v2
        r = r2
    return v, r

def factor(ts):
    if isinstance(ts[0], float):
        return ts[0], ts[1:]
    v, r = expr(ts[1:])
    return v, r[1:]

print(expr(lex(sys.argv[1]))[0])
