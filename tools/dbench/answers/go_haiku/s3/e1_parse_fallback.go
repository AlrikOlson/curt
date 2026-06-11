package main

import (
	"fmt"
	"strconv"
)

func main() {
	inputs := []string{"12", "x", "7", "-", "30"}
	total := 0
	failed := 0

	for _, input := range inputs {
		value, err := strconv.Atoi(input)
		if err != nil {
			failed++
		} else {
			total += value
		}
	}

	fmt.Println(total)
	fmt.Println(failed)
}
