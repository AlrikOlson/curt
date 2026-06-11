l = open("users.txt").readlines()
n = None
a = -1
for x in l:
    p = x.strip().split()
    if len(p) == 3:
        try:
            g = int(p[2])
            if g > a:
                a = g
                n = p[1]
        except:
            pass
if n:
    print(f"{n} {a}")
