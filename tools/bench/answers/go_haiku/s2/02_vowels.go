package main

import (
	"fmt"
)

func main() {
	s := "the quick brown fox jumps over the lazy dog"
	vowels := "aeiou"
	count := 0
	for _, ch := range s {
		for _, v := range vowels {
			if ch == v {
				count++
				break
			}
		}
	}
	fmt.Println(count)
}
