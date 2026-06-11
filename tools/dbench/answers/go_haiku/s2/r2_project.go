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
	data, _ := os.ReadFile("orders.json")
	var orders []Order
	json.Unmarshal(data, &orders)

	var selected []Order
	for _, o := range orders {
		if o.Amt > 25 {
			selected = append(selected, o)
		}
	}

	sort.Slice(selected, func(i, j int) bool {
		return selected[i].Amt > selected[j].Amt
	})

	var ids []string
	for _, o := range selected {
		ids = append(ids, fmt.Sprintf("%d", o.Id))
	}

	fmt.Println(strings.Join(ids, ","))
}
