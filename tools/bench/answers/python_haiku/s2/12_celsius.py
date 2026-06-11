temps_c = [12.5, 30.0, -5.0]
temps_f = []
for c in temps_c:
    f = c * 9 / 5 + 32
    temps_f.append(f)
print(max(temps_f))
