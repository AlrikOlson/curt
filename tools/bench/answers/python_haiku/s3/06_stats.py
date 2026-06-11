lst = [4, 8, 15, 16, 23]
min_val = lst[0]
max_val = lst[0]
sum_val = 0
for x in lst:
    if x < min_val:
        min_val = x
    if x > max_val:
        max_val = x
    sum_val += x
mean = sum_val / len(lst)
print(min_val)
print(max_val)
print(mean)
