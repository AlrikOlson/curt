package main

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"
)

type Order struct {
	ID     int     `json:"id"`
	Amt    float64 `json:"amt"`
	Status string  `json:"status"`
}

func main() {
	data, err := os.ReadFile("orders.json")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var orders []Order
	if err := json.Unmarshal(data, &orders); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var filtered []Order
	for _, o := range orders {
		if o.Amt > 25 {
			filtered = append(filtered, o)
		}
	}
	sort.Slice(filtered, func(i, j int) bool {
		return filtered[i].Amt > filtered[j].Amt
	})
	ids := make([]string, len(filtered))
	for i, o := range filtered {
		ids[i] = strconv.Itoa(o.ID)
	}
	fmt.Println(strings.Join(ids, ","))
}
