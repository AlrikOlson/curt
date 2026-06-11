import json

def read_cfg(path):
    try:
        with open(path) as f:
            return json.load(f)
    except Exception:
        return {}

print(read_cfg("missing.cfg").get("name", "default"))
print(read_cfg("app.cfg").get("port", 0))
