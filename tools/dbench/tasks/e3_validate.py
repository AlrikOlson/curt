ages = [34, -2, 19, 150, 42, 27]
ok = [a for a in ages if 0 <= a <= 120]
print(len(ok))
print(len(ages) - len(ok))
print(sum(ok))
