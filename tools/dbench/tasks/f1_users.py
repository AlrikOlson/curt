import sys
count = 0
total_age = 0
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) != 3:
            continue
        try:
            age = int(parts[2])
        except ValueError:
            continue
        count += 1
        total_age += age
print(count)
print(total_age // count)
