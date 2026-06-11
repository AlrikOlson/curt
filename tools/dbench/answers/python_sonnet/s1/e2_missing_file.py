import json

try:
    with open("missing.cfg") as f:
        data = json.load(f)
    print(data.get("name", "default"))
except Exception:
    print("default")

try:
    with open("app.cfg") as f:
        data = json.load(f)
    print(data.get("port", 0))
except Exception:
    print(0)
