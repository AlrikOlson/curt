s = "a=1,b=22,c=333"
total = 0
for pair in s.split(","):
    key, val = pair.split("=")
    total += int(val)
print(total)
