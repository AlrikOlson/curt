def show(v):
    if isinstance(v, float): return f"num {v}"
    return f"sym {v}"
print(show(2.5))
print(show("ok"))
