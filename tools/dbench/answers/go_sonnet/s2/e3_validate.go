package main

import "fmt"

func main() {
	ages := []int{34, -2, 19, 150, 42, 27}
	validCount, invalidCount, sumValid := 0, 0, 0
	for _, a := range ages {
		if a >= 0 && a <= 120 {
			validCount++
			sumValid += a
		} else {
			invalidCount++
		}
	}
	fmt.Println(validCount)
	fmt.Println(invalidCount)
	fmt.Println(sumValid)
}
