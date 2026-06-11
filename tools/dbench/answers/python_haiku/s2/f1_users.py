lines = open("users.txt").readlines()
users = []
for line in lines:
    parts = line.strip().split()
    if len(parts) == 3:
        try:
            users.append(int(parts[2]))
        except ValueError:
            pass
print(len(users))
print(sum(users) // len(users) if users else 0)
