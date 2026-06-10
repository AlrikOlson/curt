from functools import reduce
print(reduce(lambda acc, x: acc + x, [1,2,3,4], 0))
