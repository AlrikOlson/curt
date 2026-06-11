lst = [4, 8, 15, 16, 23]
minimum = lst[0]
maximum = lst[0]
total = 0
for num in lst:
    if num < minimum:
        minimum = num
    if num > maximum:
        maximum = num
    total += num
mean = total / len(lst)
print(minimum)
print(maximum)
print(mean)
