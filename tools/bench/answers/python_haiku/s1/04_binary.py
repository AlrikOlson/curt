numbers = [13, 64]
for num in numbers:
    binary = ""
    n = num
    if n == 0:
        binary = "0"
    else:
        while n > 0:
            binary = str(n % 2) + binary
            n = n // 2
    print(binary)
