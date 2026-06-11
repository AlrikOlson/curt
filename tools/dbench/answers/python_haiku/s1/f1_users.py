with open("users.txt") as f:
    lines = f.readlines()
valid_users = []
for line in lines:
    parts = line.strip().split()
    if len(parts) == 3:
        try:
            age = int(parts[2])
            valid_users.append(age)
        except ValueError:
            pass
print(len(valid_users))
if valid_users:
    print(sum(valid_users) // len(valid_users))
else:
    print(0)
