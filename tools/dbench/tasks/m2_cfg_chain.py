import json
with open("app.cfg") as f:
    cfg = json.load(f)
host = cfg.get("host", "localhost")
mode = "debug" if cfg.get("debug", False) else "prod"
print(f"{mode} {host}:{cfg['port']}")
