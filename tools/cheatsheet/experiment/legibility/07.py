def safediv(a, b):
    return "err" if b == 0 else str(a // b)
print(safediv(9, 2))
print(safediv(5, 0))
