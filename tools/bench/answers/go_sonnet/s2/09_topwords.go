package main

import (
	"fmt"
	"strings"
)

func main() {
	sentence := "the cat sat on the mat with the cat"
	words := strings.Fields(sentence)
	counts := make(map[string]int)
	order := []string{}
	for _, w := range words {
		if counts[w] == 0 {
			order = append(order, w)
		}
		counts[w]++
	}
	top1, top2 := "", ""
	cnt1, cnt2 := 0, 0
	for _, w := range order {
		c := counts[w]
		if c > cnt1 {
			top2, cnt2 = top1, cnt1
			top1, cnt1 = w, c
		} else if c > cnt2 {
			top2, cnt2 = w, c
		}
	}
	fmt.Println(top1, cnt1)
	fmt.Println(top2, cnt2)
}
