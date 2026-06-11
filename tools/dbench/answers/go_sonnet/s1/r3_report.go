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
	cfgData, err := os.ReadFile("app.cfg")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var cfg map[string]interface{}
	if err := json.Unmarshal(cfgData, &cfg); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	name, _ := cfg["name"].(string)

	ordData, err := os.ReadFile("orders.json")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var orders []Order
	if err := json.Unmarshal(ordData, &orders); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	paidCount := 0
	totalPaid := 0.0
	for _, o := range orders {
		if o.Status == "paid" {
			paidCount++
			totalPaid += o.Amt
		}
	}
	fmt.Printf("%s: %d paid, total %g\n", name, paidCount, totalPaid)
}
