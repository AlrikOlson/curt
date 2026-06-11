def hash(s):
    h = 14695981039346656037
    for b in s.encode():
        h = ((h ^ b) * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return h
print(hash("curt"))
