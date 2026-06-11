package main

import (
	"fmt"
)

func main() {
	nums := []int{13, 64}
	for _, n := range nums {
		var binary string
		if n == 0 {
			binary = "0"
		} else {
			for n > 0 {
				if n%2 == 0 {
					binary = "0" + binary
				} else {
					binary = "1" + binary
				}
				n /= 2
			}
		}
		fmt.Println(binary)
	}
}
