from collections import defaultdict
sales = [{"city":"NY","amt":50}, {"city":"LA","amt":30}, {"city":"NY","amt":20}]
g = defaultdict(int)
for s in sales: g[s["city"]] += s["amt"]
for k, v in g.items(): print(k, v)
