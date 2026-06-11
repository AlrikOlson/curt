import json

cfg = json.load(open("app.cfg"))
mode = "debug" if cfg.get("debug") else "prod"
host = cfg.get("host", "localhost")
port = cfg.get("port", 0)
print(f"{mode} {host}:{port}")
