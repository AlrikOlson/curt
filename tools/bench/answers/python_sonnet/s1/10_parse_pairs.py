s = "a=1,b=22,c=333"
pairs = s.split(",")
total = 0
for pair in pairs:
    k, v = pair.split("=")
    total += int(v)
print(total)
