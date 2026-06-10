def steps(n):
    k = 0
    while n != 1:
        n = n // 2 if n % 2 == 0 else 3*n + 1
        k += 1
    return k
print(steps(27))
