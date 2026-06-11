import json

with open("orders.json") as f:
    orders = json.load(f)

paid = [o for o in orders if o["status"] == "paid"]
open_ = [o for o in orders if o["status"] == "open"]
print(sum(o["amt"] for o in paid))
print(len(open_))
