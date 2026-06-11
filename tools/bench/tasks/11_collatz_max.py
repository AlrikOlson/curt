best_n, best_steps = 0, -1
for n in range(1, 11):
    k, m = 0, n
    while m != 1:
        m = m // 2 if m % 2 == 0 else 3 * m + 1
        k += 1
    if k > best_steps:
        best_n, best_steps = n, k
print(f"{best_n} {best_steps}")
