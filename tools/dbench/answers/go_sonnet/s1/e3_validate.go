package main

import "fmt"

func main() {
	ages := []int{34, -2, 19, 150, 42, 27}
	valid := 0
	invalid := 0
	sum := 0
	for _, age := range ages {
		if age >= 0 && age <= 120 {
			valid++
			sum += age
		} else {
			invalid++
		}
	}
	fmt.Println(valid)
	fmt.Println(invalid)
	fmt.Println(sum)
}
