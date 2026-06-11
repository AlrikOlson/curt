package main

import (
	"fmt"
)

func main() {
	ages := []int{34, -2, 19, 150, 42, 27}
	validCount := 0
	invalidCount := 0
	validSum := 0

	for _, age := range ages {
		if age >= 0 && age <= 120 {
			validCount++
			validSum += age
		} else {
			invalidCount++
		}
	}

	fmt.Println(validCount)
	fmt.Println(invalidCount)
	fmt.Println(validSum)
}
