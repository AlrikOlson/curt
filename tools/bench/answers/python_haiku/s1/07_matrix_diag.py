matrix = [[5, 1, 2], [3, 6, 4], [7, 8, 9]]
diag_sum = 0
for i in range(3):
    diag_sum += matrix[i][i]
print(diag_sum)
