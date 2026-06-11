package main

import "fmt"

func main() {
	m := [3][3]int{{5, 1, 2}, {3, 6, 4}, {7, 8, 9}}
	sum := 0
	for i := 0; i < 3; i++ {
		sum += m[i][i]
	}
	fmt.Println(sum)
}
