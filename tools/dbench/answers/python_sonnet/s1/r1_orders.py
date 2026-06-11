import json

with open("orders.json") as f:
    orders = json.load(f)

paid_total = sum(o["amt"] for o in orders if o["status"] == "paid")
open_count = sum(1 for o in orders if o["status"] == "open")
print(paid_total)
print(open_count)
