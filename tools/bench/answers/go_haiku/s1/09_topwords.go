package main

import (
	"fmt"
	"sort"
	"strings"
)

func main() {
	s := "the cat sat on the mat with the cat"
	words := strings.Fields(s)
	counts := make(map[string]int)
	for _, word := range words {
		counts[word]++
	}
	type pair struct {
		word  string
		count int
	}
	var pairs []pair
	for word, count := range counts {
		pairs = append(pairs, pair{word, count})
	}
	sort.Slice(pairs, func(i, j int) bool {
		return pairs[i].count > pairs[j].count
	})
	for i := 0; i < 2 && i < len(pairs); i++ {
		fmt.Printf("%s %d\n", pairs[i].word, pairs[i].count)
	}
}
