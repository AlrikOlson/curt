s = "a=1,b=22,c=333"
total = sum(int(pair.split("=")[1]) for pair in s.split(","))
print(total)
