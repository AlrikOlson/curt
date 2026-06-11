import json

with open("orders.json") as f:
    orders = json.load(f)

ids = [o["id"] for o in sorted(orders, key=lambda o: o["amt"], reverse=True) if o["amt"] > 25]
print(",".join(map(str, ids)))
