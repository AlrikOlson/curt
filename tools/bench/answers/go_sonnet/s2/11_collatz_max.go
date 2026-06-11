package main

import "fmt"

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
	maxN, maxSteps := 0, -1
	for n := 1; n <= 10; n++ {
		s := collatzSteps(n)
		if s > maxSteps {
			maxSteps = s
			maxN = n
		}
	}
	fmt.Println(maxN, maxSteps)
}
