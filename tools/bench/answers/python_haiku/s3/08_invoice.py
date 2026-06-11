items = [
    ("widget", 4, 2.5),
    ("gizmo", 2, 7.25),
    ("bolt", 10, 0.1)
]
grand_total = 0
max_item = None
max_line_total = 0
for name, qty, price in items:
    line_total = qty * price
    grand_total += line_total
    if line_total > max_line_total:
        max_line_total = line_total
        max_item = name
print(grand_total)
print(max_item)
