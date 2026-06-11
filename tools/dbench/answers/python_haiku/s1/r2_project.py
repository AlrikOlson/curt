import json
with open("orders.json") as f:
    orders = json.load(f)
filtered = [o for o in orders if o["amt"] > 25]
filtered.sort(key=lambda x: x["amt"], reverse=True)
ids = [str(o["id"]) for o in filtered]
print(",".join(ids))
