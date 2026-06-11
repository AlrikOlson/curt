names = []
with open("users.txt") as f:
    for line in f:
        p = line.split()
        if len(p) == 3:
            try:
                if int(p[2]) > 25:
                    names.append(p[1])
            except ValueError:
                pass
print(" ".join(names))
