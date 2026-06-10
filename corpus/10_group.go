package main

import "fmt"

type Sale struct {
	City string
	Amt  int
}

func main() {
	sales := []Sale{{"NY", 50}, {"LA", 30}, {"NY", 20}}
	g := map[string]int{}
	for _, s := range sales {
		g[s.City] += s.Amt
	}
	for k, v := range g {
		fmt.Println(k, v)
	}
}
