package main

import "fmt"

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
	grand := 0.0
	maxName := ""
	maxTotal := 0.0
	for _, item := range items {
		total := float64(item.qty) * item.price
		grand += total
		if total > maxTotal {
			maxTotal = total
			maxName = item.name
		}
	}
	fmt.Println(grand)
	fmt.Println(maxName)
}
