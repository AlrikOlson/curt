package main

import (
	"fmt"
	"strconv"
)

func main() {
	values := []string{"12", "x", "7", "-", "30"}
	sum := 0
	unparseable := 0
	for _, v := range values {
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
