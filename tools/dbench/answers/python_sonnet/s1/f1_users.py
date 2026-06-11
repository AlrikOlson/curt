users = []
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) == 3:
            try:
                users.append(int(parts[2]))
            except ValueError:
                pass

print(len(users))
print(sum(users) // len(users))
