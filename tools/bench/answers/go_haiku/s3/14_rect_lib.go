package main

import (
	"fmt"
)

type Rectangle struct {
	width  float64
	height float64
}

func (r Rectangle) area() float64 {
	return r.width * r.height
}

func (r Rectangle) perimeter() float64 {
	return 2 * (r.width + r.height)
}

func (r Rectangle) scale(factor float64) Rectangle {
	return Rectangle{r.width * factor, r.height * factor}
}

func main() {
	rect := Rectangle{3.0, 4.0}
	fmt.Println(rect.area())
	fmt.Println(rect.perimeter())
	scaled := rect.scale(2.0)
	fmt.Println(scaled.area())
}
