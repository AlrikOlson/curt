import json

cfg = json.load(open("app.cfg"))
orders = json.load(open("orders.json"))
paid = [o for o in orders if o["status"] == "paid"]
total = sum(o["amt"] for o in paid)
print(f"{cfg['name']}: {len(paid)} paid, total {total}")
