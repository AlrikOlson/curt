import json
cfg = json.load(open("app.cfg"))
m = "debug" if cfg.get("debug") else "prod"
h = cfg.get("host", "localhost")
print(f"{m} {h}:{cfg['port']}")
