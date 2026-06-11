names = []
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) != 3:
            continue
        try:
            age = int(parts[2])
        except ValueError:
            continue
        if age > 25:
            names.append(parts[1])
print(" ".join(names))
