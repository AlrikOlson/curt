lines = open("users.txt").readlines()
result = []
for line in lines:
    parts = line.strip().split()
    if len(parts) == 3:
        try:
            if int(parts[2]) > 25:
                result.append(parts[1])
        except ValueError:
            pass
print(" ".join(result))
