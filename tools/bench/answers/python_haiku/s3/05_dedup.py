lst = [3, 1, 3, 2, 1, 4]
seen = set()
result = []
for x in lst:
    if x not in seen:
        seen.add(x)
        result.append(x)
print(" ".join(str(x) for x in result))
