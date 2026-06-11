package main

import "fmt"

func main() {
	temps := []float64{12.5, 30.0, -5.0}
	max := 0.0
	for i, c := range temps {
		f := c*9.0/5.0 + 32.0
		if i == 0 || f > max {
			max = f
		}
	}
	fmt.Println(max)
}
