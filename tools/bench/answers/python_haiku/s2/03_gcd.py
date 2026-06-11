a, b = 252, 105
while b != 0:
    temp = b
    b = a % b
    a = temp
print(a)
