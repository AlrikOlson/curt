def describe(v):
    if isinstance(v, bool):
        return "?"
    if isinstance(v, int):
        return f"int {v + 1}"
    if isinstance(v, float):
        return f"float {v}"
    return f"str {len(v)}"
print(describe(42))
print(describe(3.5))
print(describe("abc"))
