l = open("users.txt").readlines()
u = []
for x in l:
    p = x.strip().split()
    if len(p) == 3:
        try:
            u.append(int(p[2]))
        except:
            pass
print(len(u))
print(sum(u) // len(u) if u else 0)
