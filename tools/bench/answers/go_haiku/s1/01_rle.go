package main

import (
	"fmt"
	"strconv"
)

func main() {
	s := "aaabbcccca"
	var result string
	for i := 0; i < len(s); i++ {
		char := s[i]
		count := 1
		for i+1 < len(s) && s[i+1] == char {
			count++
			i++
		}
		result += string(char) + strconv.Itoa(count)
	}
	fmt.Println(result)
}
