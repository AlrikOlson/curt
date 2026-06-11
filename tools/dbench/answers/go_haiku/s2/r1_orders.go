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
	data, _ := os.ReadFile("orders.json")
	var orders []Order
	json.Unmarshal(data, &orders)

	paid := 0.0
	open := 0

	for _, o := range orders {
		if o.Status == "paid" {
			paid += o.Amt
		} else if o.Status == "open" {
			open++
		}
	}

	fmt.Println(paid)
	fmt.Println(open)
}
