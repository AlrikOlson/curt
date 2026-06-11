for n in (13, 64):
    b = ""
    while n > 0:
        b = str(n % 2) + b
        n //= 2
    print(b)
