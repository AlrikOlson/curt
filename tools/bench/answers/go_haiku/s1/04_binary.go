package main

import "fmt"

func main() {
	nums := []int{13, 64}
	for _, num := range nums {
		if num == 0 {
			fmt.Println("0")
		} else {
			var binary string
			n := num
			for n > 0 {
				binary = string(rune('0' + n%2)) + binary
				n /= 2
			}
			fmt.Println(binary)
		}
	}
}
