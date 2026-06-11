package main

import (
	"fmt"
	"strconv"
	"strings"
)

func main() {
	nums := []int{3, 1, 3, 2, 1, 4}
	seen := make(map[int]bool)
	var result []string
	for _, num := range nums {
		if !seen[num] {
			seen[num] = true
			result = append(result, strconv.Itoa(num))
		}
	}
	fmt.Println(strings.Join(result, " "))
}
