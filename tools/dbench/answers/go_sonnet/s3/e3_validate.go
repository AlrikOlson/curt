package main

import "fmt"

func main() {
	ages := []int{34, -2, 19, 150, 42, 27}
	nValid, nInvalid, sumValid := 0, 0, 0
	for _, a := range ages {
		if a >= 0 && a <= 120 {
			nValid++
			sumValid += a
		} else {
			nInvalid++
		}
	}
	fmt.Println(nValid)
	fmt.Println(nInvalid)
	fmt.Println(sumValid)
}
