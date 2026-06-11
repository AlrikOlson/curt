items = [("widget", 4, 2.5), ("gizmo", 2, 7.25), ("bolt", 10, 0.1)]
grand_total = 0
best_name = ""
best_total = -1
for name, qty, price in items:
    line_total = qty * price
    grand_total += line_total
    if line_total > best_total:
        best_total = line_total
        best_name = name
print(grand_total)
print(best_name)
