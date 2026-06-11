package main

import (
	"fmt"
)

func main() {
	nums := []int{13, 64}
	for _, n := range nums {
		if n == 0 {
			fmt.Println("0")
			continue
		}
		var binary string
		x := n
		for x > 0 {
			if x%2 == 1 {
				binary = "1" + binary
			} else {
				binary = "0" + binary
			}
			x /= 2
		}
		fmt.Println(binary)
	}
}
