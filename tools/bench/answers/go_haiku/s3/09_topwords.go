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
	for _, word := range words {
		freq[word]++
	}
	type kv struct {
		word  string
		count int
	}
	var pairs []kv
	for word, count := range freq {
		pairs = append(pairs, kv{word, count})
	}
	sort.Slice(pairs, func(i, j int) bool {
		return pairs[i].count > pairs[j].count
	})
	for i := 0; i < 2 && i < len(pairs); i++ {
		fmt.Printf("%s %d\n", pairs[i].word, pairs[i].count)
	}
}
