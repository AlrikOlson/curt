import json
o = json.load(open("orders.json"))
s = sorted([x for x in o if x["amt"] > 25], key=lambda x: -x["amt"])
print(",".join(str(x["id"]) for x in s))
