package main

import (
	"fmt"
	"strconv"
)

func main() {
	values := []string{"12", "x", "7", "-", "30"}
	sum := 0
	bad := 0
	for _, s := range values {
		n, err := strconv.Atoi(s)
		if err != nil {
			bad++
			continue
		}
		sum += n
	}
	fmt.Println(sum)
	fmt.Println(bad)
}
