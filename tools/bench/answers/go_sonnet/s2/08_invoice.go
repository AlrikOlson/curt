package main

import "fmt"

func main() {
	type Item struct {
		name  string
		qty   int
		price float64
	}
	items := []Item{
		{"widget", 4, 2.5},
		{"gizmo", 2, 7.25},
		{"bolt", 10, 0.1},
	}
	total := 0.0
	maxTotal := -1.0
	maxName := ""
	for _, item := range items {
		lineTotal := float64(item.qty) * item.price
		total += lineTotal
		if lineTotal > maxTotal {
			maxTotal = lineTotal
			maxName = item.name
		}
	}
	fmt.Println(total)
	fmt.Println(maxName)
}
