import json

with open("app.cfg") as f:
    name = json.load(f)["name"]
with open("orders.json") as f:
    orders = json.load(f)

paid = [o for o in orders if o["status"] == "paid"]
print(f"{name}: {len(paid)} paid, total {sum(o['amt'] for o in paid)}")
