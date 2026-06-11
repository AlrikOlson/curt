def describe(v):
    if isinstance(v, bool):
        return f"int {v + 1}"
    elif isinstance(v, int):
        return f"int {v + 1}"
    elif isinstance(v, float):
        return f"float {v}"
    elif isinstance(v, str):
        return f"str {len(v)}"

print(describe(42))
print(describe(3.5))
print(describe("abc"))
