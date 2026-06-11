def describe(value):
    if isinstance(value, int):
        return f"int {value + 1}"
    elif isinstance(value, float):
        return f"float {value}"
    elif isinstance(value, str):
        return f"str {len(value)}"
print(describe(42))
print(describe(3.5))
print(describe("abc"))
