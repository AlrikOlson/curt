def describe(v):
    if isinstance(v, bool):
        return "int " + str(int(v) + 1)
    elif isinstance(v, int):
        return "int " + str(v + 1)
    elif isinstance(v, float):
        return "float " + str(v)
    elif isinstance(v, str):
        return "str " + str(len(v))

print(describe(42))
print(describe(3.5))
print(describe("abc"))
