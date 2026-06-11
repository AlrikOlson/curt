package main

import "fmt"

func toBinary(n int) string {
	if n == 0 {
		return "0"
	}
	bits := ""
	for n > 0 {
		if n%2 == 1 {
			bits = "1" + bits
		} else {
			bits = "0" + bits
		}
		n /= 2
	}
	return bits
}

func main() {
	fmt.Println(toBinary(13))
	fmt.Println(toBinary(64))
}
