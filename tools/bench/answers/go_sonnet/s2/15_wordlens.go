package main

import (
	"fmt"
	"strings"
)

func main() {
	sentence := "pack my box with five dozen liquor jugs"
	words := strings.Fields(sentence)
	longest := ""
	totalLen := 0
	for _, w := range words {
		totalLen += len(w)
		if len(w) > len(longest) {
			longest = w
		}
	}
	fmt.Println(longest)
	fmt.Println(totalLen / len(words))
}
