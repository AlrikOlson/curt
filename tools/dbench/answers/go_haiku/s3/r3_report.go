package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	cfgBuf, _ := os.ReadFile("app.cfg")
	var cfg map[string]interface{}
	json.Unmarshal(cfgBuf, &cfg)
	name := cfg["name"]

	ordBuf, _ := os.ReadFile("orders.json")
	var orders []map[string]interface{}
	json.Unmarshal(ordBuf, &orders)

	paidCnt := 0
	paidAmt := 0.0

	for _, ord := range orders {
		if status, ok := ord["status"].(string); ok && status == "paid" {
			paidCnt++
			if amt, ok := ord["amt"].(float64); ok {
				paidAmt += amt
			}
		}
	}

	fmt.Printf("%v: %d paid, total %v\n", name, paidCnt, paidAmt)
}
