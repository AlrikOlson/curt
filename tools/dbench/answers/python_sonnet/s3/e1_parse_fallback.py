def try_int(v):
    try:
        return int(v), False
    except ValueError:
        return 0, True

results = [try_int(v) for v in ["12", "x", "7", "-", "30"]]
print(sum(r[0] for r in results))
print(sum(1 for r in results if r[1]))
