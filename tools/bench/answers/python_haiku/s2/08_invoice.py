items = [
    ("widget", 4, 2.5),
    ("gizmo", 2, 7.25),
    ("bolt", 10, 0.1)
]
total = 0
max_item = None
max_total = 0
for name, qty, price in items:
    line_total = qty * price
    total += line_total
    if line_total > max_total:
        max_total = line_total
        max_item = name
print(total)
print(max_item)
