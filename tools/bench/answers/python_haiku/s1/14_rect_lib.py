class Rectangle:
    def __init__(self, width, height):
        self.width = width
        self.height = height

def area(rect):
    return rect.width * rect.height

def perimeter(rect):
    return 2 * (rect.width + rect.height)

def scale(rect, factor):
    return Rectangle(rect.width * factor, rect.height * factor)

rect = Rectangle(3.0, 4.0)
print(area(rect))
print(perimeter(rect))
scaled = scale(rect, 2.0)
print(area(scaled))
