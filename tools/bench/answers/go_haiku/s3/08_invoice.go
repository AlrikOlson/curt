package main

import (
	"fmt"
)

func main() {
	type Item struct {
		name      string
		qty       int
		unitPrice float64
	}
	items := []Item{
		{"widget", 4, 2.5},
		{"gizmo", 2, 7.25},
		{"bolt", 10, 0.1},
	}
	var total float64
	var maxItem Item
	var maxLineTotal float64
	for _, item := range items {
		lineTotal := float64(item.qty) * item.unitPrice
		total += lineTotal
		if lineTotal > maxLineTotal {
			maxLineTotal = lineTotal
			maxItem = item
		}
	}
	fmt.Println(total)
	fmt.Println(maxItem.name)
}
