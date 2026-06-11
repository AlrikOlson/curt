ages = [34, -2, 19, 150, 42, 27]
valid_count = 0
invalid_count = 0
valid_sum = 0
for age in ages:
    if 0 <= age <= 120:
        valid_count += 1
        valid_sum += age
    else:
        invalid_count += 1
print(valid_count)
print(invalid_count)
print(valid_sum)
