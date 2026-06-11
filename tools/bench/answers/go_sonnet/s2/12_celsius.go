package main

import "fmt"

func main() {
	temps := []float64{12.5, 30.0, -5.0}
	maxF := -1e18
	for _, c := range temps {
		f := c*9.0/5.0 + 32
		if f > maxF {
			maxF = f
		}
	}
	fmt.Println(maxF)
}
