s = "a=1,b=22,c=333"
print(sum(int(p.split("=")[1]) for p in s.split(",")))
