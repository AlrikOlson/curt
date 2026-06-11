users = []
with open("users.txt") as f:
    for line in f:
        p = line.split()
        if len(p) == 3:
            try:
                users.append((p[1], int(p[2])))
            except ValueError:
                pass
name, age = max(users, key=lambda x: x[1])
print(f"{name} {age}")
