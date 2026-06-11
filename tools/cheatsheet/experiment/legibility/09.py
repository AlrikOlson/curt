s = "14,3,5"
acc = 100
for x in s.split(","):
    acc -= int(x)
print(acc)
