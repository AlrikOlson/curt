ages = []
with open("users.txt") as f:
    for line in f:
        p = line.split()
        if len(p) == 3:
            try:
                ages.append(int(p[2]))
            except ValueError:
                pass
print(len(ages))
print(sum(ages) // len(ages))
