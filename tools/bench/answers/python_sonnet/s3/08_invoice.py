items = [("widget", 4, 2.5), ("gizmo", 2, 7.25), ("bolt", 10, 0.1)]
totals = [(name, qty * price) for name, qty, price in items]
grand = sum(t for _, t in totals)
best = max(totals, key=lambda x: x[1])
print(grand)
print(best[0])
