package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	f, _ := os.Open("users.txt")
	defer f.Close()

	sc := bufio.NewScanner(f)
	var adults []string

	for sc.Scan() {
		parts := strings.Fields(sc.Text())
		if len(parts) != 3 {
			continue
		}
		age, err := strconv.Atoi(parts[2])
		if err != nil {
			continue
		}
		if age > 25 {
			adults = append(adults, parts[1])
		}
	}

	fmt.Println(strings.Join(adults, " "))
}
