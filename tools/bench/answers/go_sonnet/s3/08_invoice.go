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
	maxLine := 0.0
	maxName := ""
	for _, it := range items {
		line := it.qty * it.price
		total += line
		if line > maxLine {
			maxLine = line
			maxName = it.name
		}
	}
	fmt.Println(total)
	fmt.Println(maxName)
}
