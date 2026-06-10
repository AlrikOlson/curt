import json
def load(p): return json.load(open(p))
try:
    cfg = load("app.cfg")
except Exception:
    cfg = {}
print(cfg.get("port", 8080))
