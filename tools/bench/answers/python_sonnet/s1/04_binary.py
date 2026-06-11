for n in [13, 64]:
    bits = []
    x = n
    while x > 0:
        bits.append(str(x % 2))
        x //= 2
    print("".join(reversed(bits)))
