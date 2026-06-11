items = [
    {"name": "widget", "qty": 4, "price": 2.5},
    {"name": "gizmo", "qty": 2, "price": 7.25},
    {"name": "bolt", "qty": 10, "price": 0.1},
]
total = sum(it["qty"] * it["price"] for it in items)
print(total)
dearest = max(items, key=lambda it: it["qty"] * it["price"])
print(dearest["name"])
