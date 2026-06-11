a = [34, -2, 19, 150, 42, 27]
v = sum(1 for x in a if 0 <= x <= 120)
i = sum(1 for x in a if x < 0 or x > 120)
s = sum(x for x in a if 0 <= x <= 120)
print(v)
print(i)
print(s)
