package main

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strconv"
)

func main() {
	data, _ := os.ReadFile("orders.json")
	var orders []map[string]interface{}
	json.Unmarshal(data, &orders)

	type orderItem struct {
		id  int
		amt float64
	}

	var filtered []orderItem
	for _, order := range orders {
		if amt, ok := order["amt"].(float64); ok && amt > 25 {
			if id, ok := order["id"].(float64); ok {
				filtered = append(filtered, orderItem{int(id), amt})
			}
		}
	}

	sort.Slice(filtered, func(i, j int) bool {
		return filtered[i].amt > filtered[j].amt
	})

	var ids []string
	for _, item := range filtered {
		ids = append(ids, strconv.Itoa(item.id))
	}

	var result string
	for i, id := range ids {
		if i > 0 {
			result += ","
		}
		result += id
	}
	fmt.Println(result)
}
