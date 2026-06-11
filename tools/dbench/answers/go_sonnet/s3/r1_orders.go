package main

import (
	"encoding/json"
	"fmt"
	"os"
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
	var paidAmt float64
	openCount := 0
	for _, o := range orders {
		if o.Status == "paid" {
			paidAmt += o.Amt
		} else if o.Status == "open" {
			openCount++
		}
	}
	fmt.Println(paidAmt)
	fmt.Println(openCount)
}
