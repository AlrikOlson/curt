vals = ["12", "x", "7", "-", "30"]
tot = 0
err = 0
for v in vals:
    try:
        tot += int(v)
    except:
        err += 1
print(tot)
print(err)
