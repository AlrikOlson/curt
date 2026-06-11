package main

import (
	"fmt"
)

func main() {
	ages := []int{34, -2, 19, 150, 42, 27}
	validCnt := 0
	invalidCnt := 0
	validSum := 0

	for _, a := range ages {
		if a >= 0 && a <= 120 {
			validCnt++
			validSum += a
		} else {
			invalidCnt++
		}
	}

	fmt.Println(validCnt)
	fmt.Println(invalidCnt)
	fmt.Println(validSum)
}
