package main

import (
	"fmt"
	"strconv"
)

func main() {
	vals := []string{"12", "x", "7", "-", "30"}
	sum := 0
	unparseable := 0

	for _, v := range vals {
		n, err := strconv.Atoi(v)
		if err != nil {
			unparseable++
		} else {
			sum += n
		}
	}

	fmt.Println(sum)
	fmt.Println(unparseable)
}
