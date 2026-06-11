batches = [
    [3, 1],
    [4, 1, 5],
]
acc = []
for b in batches:
    acc += b
acc = acc + [9]
print(" ".join(str(x) for x in acc))
def tag(v):
    if isinstance(v, int):
        return f"i{v}"
    return f"s{v}"
for v in [7, "ok"]:
    print(tag(v))
print(sum(n * n for n in range(2, 5)))
