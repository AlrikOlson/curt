import json
name = "default"
try:
    f = open("missing.cfg")
    name = json.load(f).get("name", "default")
    f.close()
except Exception:
    pass
print(name)
port = 0
try:
    f = open("app.cfg")
    port = json.load(f).get("port", 0)
    f.close()
except Exception:
    pass
print(port)
