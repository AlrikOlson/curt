import json

orders = json.load(open("orders.json"))
result = sorted((o for o in orders if o["amt"] > 25), key=lambda o: -o["amt"])
print(",".join(str(o["id"]) for o in result))
