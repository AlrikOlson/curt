import json
with open("orders.json") as f:
    orders = json.load(f)
big = sorted((o for o in orders if o["amt"] > 25), key=lambda o: -o["amt"])
print(",".join(str(o["id"]) for o in big))
