best_name, best_age = "", -1
with open("users.txt") as f:
    for line in f:
        parts = line.split()
        if len(parts) != 3:
            continue
        try:
            age = int(parts[2])
        except ValueError:
            continue
        if age > best_age:
            best_name, best_age = parts[1], age
print(f"{best_name} {best_age}")
