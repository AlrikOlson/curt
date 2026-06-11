import json

with open("app.cfg") as f:
    cfg = json.load(f)

mode = "debug" if cfg.get("debug") else "prod"
print(f"{mode} {cfg.get('host', 'localhost')}:{cfg.get('port', 0)}")
