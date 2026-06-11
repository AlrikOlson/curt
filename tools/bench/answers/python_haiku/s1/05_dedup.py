lst = [3, 1, 3, 2, 1, 4]
seen = set()
result = []
for item in lst:
    if item not in seen:
        seen.add(item)
        result.append(item)
print(" ".join(str(x) for x in result))
