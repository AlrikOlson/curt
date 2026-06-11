import json
try:
    with open("missing.cfg") as f:
        cfg = json.load(f)
    name = cfg.get("name", "default")
except OSError:
    name = "default"
print(name)
with open("app.cfg") as f:
    cfg = json.load(f)
print(cfg.get("port", 0))
