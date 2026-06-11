l = open("users.txt").readlines()
r = []
for x in l:
    p = x.strip().split()
    if len(p) == 3:
        try:
            if int(p[2]) > 25:
                r.append(p[1])
        except:
            pass
print(" ".join(r))
