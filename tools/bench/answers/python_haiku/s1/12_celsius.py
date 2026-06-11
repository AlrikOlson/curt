celsius = [12.5, 30.0, -5.0]
fahrenheit = []
for c in celsius:
    f = c * 9 / 5 + 32
    fahrenheit.append(f)
max_f = fahrenheit[0]
for f in fahrenheit:
    if f > max_f:
        max_f = f
print(max_f)
