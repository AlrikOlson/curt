nums = [13, 64]
for n in nums:
    result = ""
    if n == 0:
        result = "0"
    else:
        while n > 0:
            result = str(n % 2) + result
            n //= 2
    print(result)
