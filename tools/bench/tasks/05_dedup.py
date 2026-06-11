xs = [3, 1, 3, 2, 1, 4]
seen = []
for x in xs:
    if x not in seen:
        seen.append(x)
print(" ".join(str(x) for x in seen))
