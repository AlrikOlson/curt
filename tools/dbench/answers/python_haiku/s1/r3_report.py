import json
with open("app.cfg") as f:
    cfg = json.load(f)
with open("orders.json") as f:
    orders = json.load(f)
name = cfg["name"]
paid_count = sum(1 for o in orders if o["status"] == "paid")
paid_total = sum(o["amt"] for o in orders if o["status"] == "paid")
print(f"{name}: {paid_count} paid, total {paid_total}")
