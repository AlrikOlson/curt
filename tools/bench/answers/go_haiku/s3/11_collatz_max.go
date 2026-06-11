package main

import (
	"fmt"
)

func collatzSteps(n int) int {
	steps := 0
	for n != 1 {
		if n%2 == 0 {
			n /= 2
		} else {
			n = 3*n + 1
		}
		steps++
	}
	return steps
}

func main() {
	maxN := 1
	maxSteps := 0
	for n := 1; n <= 10; n++ {
		steps := collatzSteps(n)
		if steps > maxSteps {
			maxSteps = steps
			maxN = n
		}
	}
	fmt.Printf("%d %d\n", maxN, maxSteps)
}
