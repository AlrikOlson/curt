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
	paidTotal := 0.0
	openCount := 0
	for _, o := range orders {
		switch o.Status {
		case "paid":
			paidTotal += o.Amt
		case "open":
			openCount++
		}
	}
	fmt.Println(paidTotal)
	fmt.Println(openCount)
}
