vals = ["12", "x", "7", "-", "30"]
total, bad = 0, 0
for v in vals:
    try:
        total += int(v)
    except ValueError:
        bad += 1
print(total)
print(bad)
