valid = []
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) == 3:
            try:
                valid.append(int(parts[2]))
            except ValueError:
                pass
print(len(valid))
print(sum(valid) // len(valid))
