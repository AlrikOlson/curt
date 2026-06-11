ages = [34, -2, 19, 150, 42, 27]
v, inv = [], []
for a in ages:
    (v if 0 <= a <= 120 else inv).append(a)
print(len(v))
print(len(inv))
print(sum(v))
