vals = ["12", "x", "7", "-", "30"]
parsed = []
bad = 0
for v in vals:
    try:
        parsed.append(int(v))
    except ValueError:
        bad += 1
print(sum(parsed))
print(bad)
