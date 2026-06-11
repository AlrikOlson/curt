package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	cfgData, _ := os.ReadFile("app.cfg")
	var cfg map[string]interface{}
	json.Unmarshal(cfgData, &cfg)
	name := cfg["name"]

	ordersData, _ := os.ReadFile("orders.json")
	var orders []map[string]interface{}
	json.Unmarshal(ordersData, &orders)

	paidCount := 0
	totalPaid := 0.0

	for _, order := range orders {
		if status, ok := order["status"].(string); ok && status == "paid" {
			paidCount++
			if amt, ok := order["amt"].(float64); ok {
				totalPaid += amt
			}
		}
	}

	fmt.Printf("%v: %d paid, total %v\n", name, paidCount, totalPaid)
}
