package main

import "fmt"

func main() {
	nums := []int{3, 1, 3, 2, 1, 4}
	seen := make(map[int]bool)
	result := []int{}
	for _, n := range nums {
		if !seen[n] {
			seen[n] = true
			result = append(result, n)
		}
	}
	for i, n := range result {
		if i > 0 {
			fmt.Print(" ")
		}
		fmt.Print(n)
	}
	fmt.Println()
}
