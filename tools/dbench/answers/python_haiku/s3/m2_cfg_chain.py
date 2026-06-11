import json
c = json.load(open("app.cfg"))
m = "debug" if c.get("debug") else "prod"
h = c.get("host", "localhost")
print(f"{m} {h}:{c['port']}")
