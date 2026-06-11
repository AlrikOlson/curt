data = ["12", "x", "7", "-", "30"]
total = 0
unparseable = 0
for item in data:
    try:
        total += int(item)
    except ValueError:
        unparseable += 1
print(total)
print(unparseable)
