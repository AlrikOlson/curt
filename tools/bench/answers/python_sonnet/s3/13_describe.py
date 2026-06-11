def describe(v):
    if isinstance(v, bool):
        return "int " + str(int(v) + 1)
    if isinstance(v, int):
        return "int " + str(v + 1)
    if isinstance(v, float):
        return "float " + str(v)
    if isinstance(v, str):
        return "str " + str(len(v))

print(describe(42))
print(describe(3.5))
print(describe("abc"))
