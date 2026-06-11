oldest = None
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) == 3:
            try:
                age = int(parts[2])
                if oldest is None or age > oldest[1]:
                    oldest = (parts[1], age)
            except ValueError:
                pass

print(f"{oldest[0]} {oldest[1]}")
