items = ["12", "x", "7", "-", "30"]
sum_val = 0
bad_count = 0
for item in items:
    try:
        sum_val += int(item)
    except ValueError:
        bad_count += 1
print(sum_val)
print(bad_count)
