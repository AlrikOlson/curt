package main

import (
	"fmt"
	"strings"
)

func main() {
	nums := []int{3, 1, 3, 2, 1, 4}
	seen := make(map[int]bool)
	var result []int
	for _, n := range nums {
		if !seen[n] {
			seen[n] = true
			result = append(result, n)
		}
	}
	var strNums []string
	for _, n := range result {
		strNums = append(strNums, fmt.Sprintf("%d", n))
	}
	fmt.Println(strings.Join(strNums, " "))
}
