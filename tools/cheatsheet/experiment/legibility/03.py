def f(v):
    if isinstance(v, int):
        return v + 1
    return len(v)
print(f(10))
print(f("abc"))
