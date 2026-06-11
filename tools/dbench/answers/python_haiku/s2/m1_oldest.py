lines = open("users.txt").readlines()
best_name = None
best_age = -1
for line in lines:
    parts = line.strip().split()
    if len(parts) == 3:
        try:
            age = int(parts[2])
            if age > best_age:
                best_age = age
                best_name = parts[1]
        except ValueError:
            pass
if best_name:
    print(f"{best_name} {best_age}")
