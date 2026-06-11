s = "a=1,b=22,c=333"
pairs = s.split(",")
total = 0
for pair in pairs:
    key, value = pair.split("=")
    total += int(value)
print(total)
