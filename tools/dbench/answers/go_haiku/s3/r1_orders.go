package main

import (
	"encoding/json"
	"fmt"
	"os"
)

type Order struct {
	Id     int     `json:"id"`
	Amt    float64 `json:"amt"`
	Status string  `json:"status"`
}

func main() {
	buf, _ := os.ReadFile("orders.json")
	var orders []Order
	json.Unmarshal(buf, &orders)

	paidTotal := 0.0
	openCount := 0

	for _, ord := range orders {
		if ord.Status == "paid" {
			paidTotal += ord.Amt
		} else if ord.Status == "open" {
			openCount++
		}
	}

	fmt.Println(paidTotal)
	fmt.Println(openCount)
}
