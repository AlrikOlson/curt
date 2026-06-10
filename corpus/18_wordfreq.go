package main

import (
	"fmt"
	"os"
	"sort"
	"strings"
)

func main() {
	data, _ := os.ReadFile(os.Args[1])
	counts := map[string]int{}
	for _, w := range strings.Fields(strings.ToLower(string(data))) {
		counts[w]++
	}
	type pair struct {
		w string
		n int
	}
	pairs := []pair{}
	for w, n := range counts {
		pairs = append(pairs, pair{w, n})
	}
	sort.Slice(pairs, func(i, j int) bool { return pairs[i].n > pairs[j].n })
	for _, p := range pairs[:10] {
		fmt.Println(p.w, p.n)
	}
}
