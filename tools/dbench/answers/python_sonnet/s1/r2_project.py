import json

with open("orders.json") as f:
    orders = json.load(f)

filtered = sorted([o for o in orders if o["amt"] > 25], key=lambda o: o["amt"], reverse=True)
print(",".join(str(o["id"]) for o in filtered))
