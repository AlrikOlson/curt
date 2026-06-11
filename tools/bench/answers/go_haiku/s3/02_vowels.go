package main

import (
	"fmt"
	"strings"
)

func main() {
	s := "the quick brown fox jumps over the lazy dog"
	vowels := "aeiou"
	count := 0
	for _, ch := range s {
		if strings.ContainsRune(vowels, ch) {
			count++
		}
	}
	fmt.Println(count)
}
