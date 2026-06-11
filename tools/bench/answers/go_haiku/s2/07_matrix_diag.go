package main

import (
	"fmt"
)

func main() {
	matrix := [][]int{
		{5, 1, 2},
		{3, 6, 4},
		{7, 8, 9},
	}
	sum := 0
	for i := 0; i < len(matrix); i++ {
		sum += matrix[i][i]
	}
	fmt.Println(sum)
}
