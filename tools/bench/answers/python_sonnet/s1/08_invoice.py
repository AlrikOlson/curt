items = [("widget", 4, 2.5), ("gizmo", 2, 7.25), ("bolt", 10, 0.1)]
grand_total = 0
best_name = None
best_line = -1
for name, qty, price in items:
    line = qty * price
    grand_total += line
    if line > best_line:
        best_line = line
        best_name = name
print(grand_total)
print(best_name)
