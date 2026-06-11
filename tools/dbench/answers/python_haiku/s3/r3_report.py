import json
c = json.load(open("app.cfg"))
o = json.load(open("orders.json"))
p = [x for x in o if x["status"] == "paid"]
print(f"{c['name']}: {len(p)} paid, total {sum(x['amt'] for x in p)}")
