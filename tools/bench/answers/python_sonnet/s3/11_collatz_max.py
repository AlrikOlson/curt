best_n, best_steps = 0, 0
for n in range(1, 11):
    x, steps = n, 0
    while x != 1:
        x = x // 2 if x % 2 == 0 else 3 * x + 1
        steps += 1
    if steps > best_steps:
        best_steps = steps
        best_n = n
print(best_n, best_steps)
