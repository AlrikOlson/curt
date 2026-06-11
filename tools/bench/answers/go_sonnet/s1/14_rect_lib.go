package main

import "fmt"

type rectangle struct {
	width  float64
	height float64
}

func area(r rectangle) float64 {
	return r.width * r.height
}

func perimeter(r rectangle) float64 {
	return 2 * (r.width + r.height)
}

func scale(r rectangle, factor float64) rectangle {
	return rectangle{r.width * factor, r.height * factor}
}

func main() {
	r := rectangle{3.0, 4.0}
	fmt.Println(area(r))
	fmt.Println(perimeter(r))
	fmt.Println(area(scale(r, 2.0)))
}
