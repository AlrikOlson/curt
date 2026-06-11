package main

import "fmt"

func main() {
	a, b := 252, 105
	for b != 0 {
		a, b = b, a%b
	}
	fmt.Println(a)
}
