package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	data, _ := os.ReadFile("orders.json")
	var orders []map[string]interface{}
	json.Unmarshal(data, &orders)

	totalPaid := 0.0
	openCount := 0

	for _, order := range orders {
		if status, ok := order["status"].(string); ok {
			if status == "paid" {
				if amt, ok := order["amt"].(float64); ok {
					totalPaid += amt
				}
			} else if status == "open" {
				openCount++
			}
		}
	}

	fmt.Println(totalPaid)
	fmt.Println(openCount)
}
