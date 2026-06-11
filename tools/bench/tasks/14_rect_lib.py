class Rect:
    def __init__(self, w, h):
        self.w = w
        self.h = h
def area(r):
    return r.w * r.h
def perim(r):
    return 2.0 * (r.w + r.h)
def scale(r, k):
    return Rect(r.w * k, r.h * k)
r = Rect(3.0, 4.0)
print(area(r))
print(perim(r))
print(area(scale(r, 2.0)))
