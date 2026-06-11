ps = [{"name": "u", "age": 3}, {"name": "v", "age": 9}, {"name": "w", "age": 6}]
top = sorted(ps, key=lambda p: p["age"], reverse=True)[:2]
print("-".join(p["name"] for p in top))
