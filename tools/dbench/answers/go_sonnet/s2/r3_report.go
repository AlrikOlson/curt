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
	cfgB, err := os.ReadFile("app.cfg")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var cfg map[string]interface{}
	if err := json.Unmarshal(cfgB, &cfg); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	name, _ := cfg["name"].(string)

	ordB, err := os.ReadFile("orders.json")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var orders []Order
	if err := json.Unmarshal(ordB, &orders); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	count := 0
	total := 0.0
	for _, o := range orders {
		if o.Status == "paid" {
			count++
			total += o.Amt
		}
	}
	fmt.Printf("%s: %d paid, total %g\n", name, count, total)
}
