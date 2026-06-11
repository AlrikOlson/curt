package main

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strings"
)

type Order struct {
	Id  int     `json:"id"`
	Amt float64 `json:"amt"`
}

func main() {
	buf, _ := os.ReadFile("orders.json")
	var orders []Order
	json.Unmarshal(buf, &orders)

	var matches []Order
	for _, ord := range orders {
		if ord.Amt > 25 {
			matches = append(matches, ord)
		}
	}

	sort.Slice(matches, func(i, j int) bool {
		return matches[i].Amt > matches[j].Amt
	})

	var idStrs []string
	for _, ord := range matches {
		idStrs = append(idStrs, fmt.Sprintf("%d", ord.Id))
	}

	fmt.Println(strings.Join(idStrs, ","))
}
