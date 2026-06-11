package main

import (
	"fmt"
	"strconv"
)

func main() {
	inputs := []string{"12", "x", "7", "-", "30"}
	total := 0
	failures := 0
	for _, v := range inputs {
		if n, err := strconv.Atoi(v); err == nil {
			total += n
		} else {
			failures++
		}
	}
	fmt.Println(total)
	fmt.Println(failures)
}
