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
	var name string
	age := -1

	for sc.Scan() {
		parts := strings.Fields(sc.Text())
		if len(parts) != 3 {
			continue
		}
		a, err := strconv.Atoi(parts[2])
		if err != nil {
			continue
		}
		if a > age {
			age = a
			name = parts[1]
		}
	}

	fmt.Printf("%s %d\n", name, age)
}
