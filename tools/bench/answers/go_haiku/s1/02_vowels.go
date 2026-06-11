package main

import (
	"fmt"
	"strings"
)

func main() {
	s := "the quick brown fox jumps over the lazy dog"
	count := 0
	vowels := "aeiou"
	for _, char := range s {
		if strings.ContainsRune(vowels, char) {
			count++
		}
	}
	fmt.Println(count)
}
