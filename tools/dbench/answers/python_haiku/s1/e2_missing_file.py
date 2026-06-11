import json
try:
    with open("missing.cfg") as f:
        config = json.load(f)
    print(config.get("name", "default"))
except Exception:
    print("default")
try:
    with open("app.cfg") as f:
        config = json.load(f)
    print(config.get("port", 0))
except Exception:
    print(0)
