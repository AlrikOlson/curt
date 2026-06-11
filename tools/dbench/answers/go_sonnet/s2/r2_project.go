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
	b, err := os.ReadFile("orders.json")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var orders []Order
	if err := json.Unmarshal(b, &orders); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var result []Order
	for _, o := range orders {
		if o.Amt > 25 {
			result = append(result, o)
		}
	}
	sort.Slice(result, func(i, j int) bool {
		return result[i].Amt > result[j].Amt
	})
	parts := make([]string, len(result))
	for i, o := range result {
		parts[i] = strconv.Itoa(o.ID)
	}
	fmt.Println(strings.Join(parts, ","))
}
