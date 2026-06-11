def half(x):
    return x // 2 if x % 2 == 0 else 3 * x + 1
print(half(half(10)))
