package main

import (
	"fmt"
	"strconv"
	"strings"
)

func main() {
	s := "a=1,b=22,c=333"
	pairs := strings.Split(s, ",")
	sum := 0
	for _, pair := range pairs {
		parts := strings.Split(pair, "=")
		val, _ := strconv.Atoi(parts[1])
		sum += val
	}
	fmt.Println(sum)
}
