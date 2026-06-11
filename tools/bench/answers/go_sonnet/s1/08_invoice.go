package main

import "fmt"

func main() {
	type item struct {
		name  string
		qty   float64
		price float64
	}
	items := []item{
		{"widget", 4, 2.5},
		{"gizmo", 2, 7.25},
		{"bolt", 10, 0.1},
	}
	total := 0.0
	maxTotal := 0.0
	maxName := ""
	for _, it := range items {
		lineTotal := it.qty * it.price
		total += lineTotal
		if lineTotal > maxTotal {
			maxTotal = lineTotal
			maxName = it.name
		}
	}
	fmt.Println(total)
	fmt.Println(maxName)
}
