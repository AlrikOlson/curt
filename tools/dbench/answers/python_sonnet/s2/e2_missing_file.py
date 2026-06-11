import json

for fname, key, default in [("missing.cfg", "name", "default"), ("app.cfg", "port", 0)]:
    try:
        with open(fname) as f:
            print(json.load(f).get(key, default))
    except Exception:
        print(default)
