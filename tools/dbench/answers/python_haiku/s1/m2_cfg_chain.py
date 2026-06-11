import json
with open("app.cfg") as f:
    cfg = json.load(f)
mode = "debug" if cfg.get("debug", False) else "prod"
host = cfg.get("host", "localhost")
port = cfg["port"]
print(f"{mode} {host}:{port}")
