package main

import (
	"fmt"
	"sort"
	"strings"
)

func main() {
	sentence := "the cat sat on the mat with the cat"
	words := strings.Fields(sentence)
	freq := make(map[string]int)
	for _, w := range words {
		freq[w]++
	}

	type wordCount struct {
		word  string
		count int
	}
	var wc []wordCount
	for w, c := range freq {
		wc = append(wc, wordCount{w, c})
	}

	sort.Slice(wc, func(i, j int) bool {
		return wc[i].count > wc[j].count
	})

	for i := 0; i < 2 && i < len(wc); i++ {
		fmt.Printf("%s %d\n", wc[i].word, wc[i].count)
	}
}
