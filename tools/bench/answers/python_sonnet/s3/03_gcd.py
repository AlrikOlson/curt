a, b = 252, 105
while b:
    a, b = b, a % b
print(a)
