items = [
    ("widget", 4, 2.5),
    ("gizmo", 2, 7.25),
    ("bolt", 10, 0.1)
]
total = 0
max_line_total = 0
max_item_name = ""
for name, qty, price in items:
    line_total = qty * price
    total += line_total
    if line_total > max_line_total:
        max_line_total = line_total
        max_item_name = name
print(total)
print(max_item_name)
