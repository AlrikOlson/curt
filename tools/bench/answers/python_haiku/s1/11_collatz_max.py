max_steps = 0
max_n = 0
for n in range(1, 11):
    current = n
    steps = 0
    while current != 1:
        if current % 2 == 0:
            current = current // 2
        else:
            current = 3 * current + 1
        steps += 1
    if steps > max_steps:
        max_steps = steps
        max_n = n
print(f"{max_n} {max_steps}")
