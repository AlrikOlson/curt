import json
with open("orders.json") as f:
    orders = json.load(f)
paid_total = 0
open_count = 0
for order in orders:
    if order["status"] == "paid":
        paid_total += order["amt"]
    elif order["status"] == "open":
        open_count += 1
print(paid_total)
print(open_count)
