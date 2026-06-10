from dataclasses import dataclass
import math
@dataclass
class Pt: x: float; y: float
def dist(a, b): return math.sqrt((a.x-b.x)**2 + (a.y-b.y)**2)
print(dist(Pt(0, 0), Pt(3, 4)))
