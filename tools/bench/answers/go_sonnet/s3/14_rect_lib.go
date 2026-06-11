package main

import "fmt"

type rect struct {
	width  float64
	height float64
}

func area(r rect) float64 {
	return r.width * r.height
}

func perimeter(r rect) float64 {
	return 2 * (r.width + r.height)
}

func scale(r rect, factor float64) rect {
	return rect{r.width * factor, r.height * factor}
}

func main() {
	r := rect{3.0, 4.0}
	fmt.Println(area(r))
	fmt.Println(perimeter(r))
	fmt.Println(area(scale(r, 2.0)))
}
