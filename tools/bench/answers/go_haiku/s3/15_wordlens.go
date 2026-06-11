package main

import (
	"fmt"
	"strings"
)

func main() {
	sentence := "pack my box with five dozen liquor jugs"
	words := strings.Fields(sentence)
	var longest string
	for _, word := range words {
		if len(word) > len(longest) {
			longest = word
		}
	}
	totalLen := 0
	for _, word := range words {
		totalLen += len(word)
	}
	avgLen := totalLen / len(words)
	fmt.Println(longest)
	fmt.Println(avgLen)
}
