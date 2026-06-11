package main

import (
	"fmt"
	"strconv"
)

func main() {
	strs := []string{"12", "x", "7", "-", "30"}
	sum := 0
	bad := 0

	for _, s := range strs {
		num, err := strconv.Atoi(s)
		if err != nil {
			bad++
		} else {
			sum += num
		}
	}

	fmt.Println(sum)
	fmt.Println(bad)
}
