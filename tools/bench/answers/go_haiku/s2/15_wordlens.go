package main

import (
	"fmt"
	"strings"
)

func main() {
	sentence := "pack my box with five dozen liquor jugs"
	words := strings.Fields(sentence)

	longest := words[0]
	for _, w := range words {
		if len(w) > len(longest) {
			longest = w
		}
	}

	totalLen := 0
	for _, w := range words {
		totalLen += len(w)
	}
	avgLen := totalLen / len(words)

	fmt.Println(longest)
	fmt.Println(avgLen)
}
