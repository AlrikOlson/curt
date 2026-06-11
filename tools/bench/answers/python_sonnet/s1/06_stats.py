lst = [4, 8, 15, 16, 23]
mn = lst[0]
mx = lst[0]
for x in lst:
    if x < mn:
        mn = x
    if x > mx:
        mx = x
mean = sum(lst) / len(lst)
print(mn)
print(mx)
print(mean)
