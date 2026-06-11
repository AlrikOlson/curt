import json
orders = json.load(open("orders.json"))
paid_amt = sum(o["amt"] for o in orders if o["status"] == "paid")
open_cnt = sum(1 for o in orders if o["status"] == "open")
print(paid_amt)
print(open_cnt)
