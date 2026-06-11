best = None
with open("users.txt") as f:
    for line in f:
        p = line.split()
        if len(p) == 3:
            try:
                age = int(p[2])
                if best is None or age > best[1]:
                    best = (p[1], age)
            except ValueError:
                pass
print(f"{best[0]} {best[1]}")
