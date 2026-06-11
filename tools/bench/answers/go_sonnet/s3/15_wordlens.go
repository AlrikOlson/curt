package main

import (
	"fmt"
	"strings"
)

func main() {
	s := "pack my box with five dozen liquor jugs"
	words := strings.Fields(s)
	longest := ""
	totalLen := 0
	for _, w := range words {
		if len(w) > len(longest) {
			longest = w
		}
		totalLen += len(w)
	}
	fmt.Println(longest)
	fmt.Println(totalLen / len(words))
}
