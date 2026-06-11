package main

import (
	"fmt"
)

func main() {
	temps := []float64{12.5, 30.0, -5.0}
	maxF := 0.0
	for i, c := range temps {
		f := c*9/5 + 32
		if i == 0 || f > maxF {
			maxF = f
		}
	}
	fmt.Println(maxF)
}
