import json
try:
    print(json.load(open("missing.cfg")).get("name", "default"))
except:
    print("default")
try:
    print(json.load(open("app.cfg")).get("port", 0))
except:
    print(0)
