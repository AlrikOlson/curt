import json

orders = json.load(open("orders.json"))
print(sum(o["amt"] for o in orders if o["status"] == "paid"))
print(sum(1 for o in orders if o["status"] == "open"))
