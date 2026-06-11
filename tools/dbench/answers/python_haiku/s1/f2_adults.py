with open("users.txt") as f:
    lines = f.readlines()
adults = []
for line in lines:
    parts = line.strip().split()
    if len(parts) == 3:
        try:
            age = int(parts[2])
            if age > 25:
                adults.append(parts[1])
        except ValueError:
            pass
print(" ".join(adults))
