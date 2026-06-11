package main

import "fmt"

type Rectangle struct {
	width  float64
	height float64
}

func area(r Rectangle) float64 {
	return r.width * r.height
}

func perimeter(r Rectangle) float64 {
	return 2 * (r.width + r.height)
}

func scale(r Rectangle, factor float64) Rectangle {
	return Rectangle{r.width * factor, r.height * factor}
}

func main() {
	r := Rectangle{3.0, 4.0}
	fmt.Println(area(r))
	fmt.Println(perimeter(r))
	r2 := scale(r, 2.0)
	fmt.Println(area(r2))
}
