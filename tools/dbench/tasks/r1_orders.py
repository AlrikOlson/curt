import json
with open("orders.json") as f:
    orders = json.load(f)
paid = [o for o in orders if o["status"] == "paid"]
print(sum(o["amt"] for o in paid))
print(len([o for o in orders if o["status"] == "open"]))
