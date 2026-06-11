package main

import (
	"fmt"
)

func main() {
	items := []struct {
		name  string
		qty   int
		price float64
	}{
		{"widget", 4, 2.5},
		{"gizmo", 2, 7.25},
		{"bolt", 10, 0.1},
	}

	total := 0.0
	maxItem := ""
	maxLineTotal := 0.0

	for _, item := range items {
		lineTotal := float64(item.qty) * item.price
		total += lineTotal
		if lineTotal > maxLineTotal {
			maxLineTotal = lineTotal
			maxItem = item.name
		}
	}

	fmt.Println(total)
	fmt.Println(maxItem)
}
