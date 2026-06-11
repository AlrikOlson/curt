def collatz_steps(n):
    steps = 0
    while n != 1:
        if n % 2 == 0:
            n //= 2
        else:
            n = 3 * n + 1
        steps += 1
    return steps

max_n = 1
max_steps = 0
for n in range(1, 11):
    steps = collatz_steps(n)
    if steps > max_steps:
        max_steps = steps
        max_n = n
print(f"{max_n} {max_steps}")
