us = [{"name":"a","score":9,"active":True}, {"name":"b","score":7,"active":False}, {"name":"c","score":8,"active":True}]
top = sorted((u for u in us if u["active"]), key=lambda u: u["score"], reverse=True)[:2]
print([u["name"] for u in top])
