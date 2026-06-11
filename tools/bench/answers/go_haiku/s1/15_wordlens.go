package main

import (
	"fmt"
	"strings"
)

func main() {
	s := "pack my box with five dozen liquor jugs"
	words := strings.Fields(s)
	longest := words[0]
	totalLen := 0
	for _, word := range words {
		if len(word) > len(longest) {
			longest = word
		}
		totalLen += len(word)
	}
	avgLen := totalLen / len(words)
	fmt.Println(longest)
	fmt.Println(avgLen)
}
