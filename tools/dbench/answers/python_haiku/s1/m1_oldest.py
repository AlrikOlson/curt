with open("users.txt") as f:
    lines = f.readlines()
oldest = None
max_age = -1
for line in lines:
    parts = line.strip().split()
    if len(parts) == 3:
        try:
            age = int(parts[2])
            if age > max_age:
                max_age = age
                oldest = parts[1]
        except ValueError:
            pass
if oldest:
    print(f"{oldest} {max_age}")
