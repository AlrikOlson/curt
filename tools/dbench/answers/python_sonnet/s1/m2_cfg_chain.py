import json

with open("app.cfg") as f:
    cfg = json.load(f)

mode = "debug" if cfg.get("debug") else "prod"
host = cfg.get("host", "localhost")
port = cfg.get("port", 0)
print(f"{mode} {host}:{port}")
