import json
orders = json.load(open("orders.json"))
big = sorted([o for o in orders if o["amt"] > 25], key=lambda x: -x["amt"])
print(",".join(str(o["id"]) for o in big))
