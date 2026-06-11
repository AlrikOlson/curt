adults = []
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) == 3:
            try:
                if int(parts[2]) > 25:
                    adults.append(parts[1])
            except ValueError:
                pass

print(" ".join(adults))
