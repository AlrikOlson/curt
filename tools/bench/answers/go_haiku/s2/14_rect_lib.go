package main

import (
	"fmt"
)

type Rectangle struct {
	width  float64
	height float64
}

func (r Rectangle) Area() float64 {
	return r.width * r.height
}

func (r Rectangle) Perimeter() float64 {
	return 2 * (r.width + r.height)
}

func (r Rectangle) Scale(factor float64) Rectangle {
	return Rectangle{r.width * factor, r.height * factor}
}

func main() {
	rect := Rectangle{3.0, 4.0}
	fmt.Println(rect.Area())
	fmt.Println(rect.Perimeter())
	scaled := rect.Scale(2.0)
	fmt.Println(scaled.Area())
}
