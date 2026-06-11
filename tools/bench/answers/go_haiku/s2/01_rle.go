package main

import (
	"fmt"
)

func main() {
	s := "aaabbcccca"
	var result string
	i := 0
	for i < len(s) {
		char := s[i]
		count := 1
		for i+count < len(s) && s[i+count] == char {
			count++
		}
		result += fmt.Sprintf("%c%d", char, count)
		i += count
	}
	fmt.Println(result)
}
