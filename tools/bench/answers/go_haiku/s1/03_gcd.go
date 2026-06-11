package main

import "fmt"

func main() {
	a := 252
	b := 105
	for b != 0 {
		temp := b
		b = a % b
		a = temp
	}
	fmt.Println(a)
}
