package main

import (
	"encoding/json"
	"fmt"
	"os"
)

type Config struct {
	Name string `json:"name"`
}

type Order struct {
	Status string  `json:"status"`
	Amt    float64 `json:"amt"`
}

func main() {
	cfgData, _ := os.ReadFile("app.cfg")
	var cfg Config
	json.Unmarshal(cfgData, &cfg)

	ordersData, _ := os.ReadFile("orders.json")
	var orders []Order
	json.Unmarshal(ordersData, &orders)

	cnt := 0
	tot := 0.0
	for _, o := range orders {
		if o.Status == "paid" {
			cnt++
			tot += o.Amt
		}
	}

	fmt.Printf("%s: %d paid, total %v\n", cfg.Name, cnt, tot)
}
