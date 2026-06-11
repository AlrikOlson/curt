class Rectangle:
    def __init__(self, width, height):
        self.width = float(width)
        self.height = float(height)

def area(r):
    return r.width * r.height

def perimeter(r):
    return 2 * (r.width + r.height)

def scale(r, factor):
    return Rectangle(r.width * factor, r.height * factor)

r = Rectangle(3.0, 4.0)
print(area(r))
print(perimeter(r))
print(area(scale(r, 2.0)))
