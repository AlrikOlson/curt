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
	var sel []Order
	for _, o := range orders {
		if o.Amt > 25 {
			sel = append(sel, o)
		}
	}
	sort.Slice(sel, func(i, j int) bool {
		return sel[i].Amt > sel[j].Amt
	})
	strs := make([]string, len(sel))
	for i, o := range sel {
		strs[i] = strconv.Itoa(o.ID)
	}
	fmt.Println(strings.Join(strs, ","))
}
