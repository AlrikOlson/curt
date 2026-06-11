ages = [34, -2, 19, 150, 42, 27]
valid = [a for a in ages if 0 <= a <= 120]
print(len(valid))
print(len(ages) - len(valid))
print(sum(valid))
