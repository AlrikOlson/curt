package main

import "fmt"

func main() {
	temps := []float64{12.5, 30.0, -5.0}
	maxF := temps[0]*9/5 + 32
	for _, c := range temps[1:] {
		f := c*9/5 + 32
		if f > maxF {
			maxF = f
		}
	}
	fmt.Println(maxF)
}
