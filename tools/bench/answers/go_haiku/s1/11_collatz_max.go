package main

import "fmt"

func main() {
	maxSteps := 0
	maxN := 0
	for n := 1; n <= 10; n++ {
		steps := 0
		x := n
		for x != 1 {
			if x%2 == 0 {
				x /= 2
			} else {
				x = 3*x + 1
			}
			steps++
		}
		if steps > maxSteps {
			maxSteps = steps
			maxN = n
		}
	}
	fmt.Printf("%d %d\n", maxN, maxSteps)
}
