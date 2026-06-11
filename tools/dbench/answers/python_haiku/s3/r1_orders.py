import json
o = json.load(open("orders.json"))
print(sum(x["amt"] for x in o if x["status"] == "paid"))
print(sum(1 for x in o if x["status"] == "open"))
