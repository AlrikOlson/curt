package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	file, _ := os.Open("users.txt")
	defer file.Close()

	scanner := bufio.NewScanner(file)
	var oldestName string
	maxAge := -1

	for scanner.Scan() {
		fields := strings.Fields(scanner.Text())
		if len(fields) != 3 {
			continue
		}
		age, err := strconv.Atoi(fields[2])
		if err != nil {
			continue
		}
		if age > maxAge {
			maxAge = age
			oldestName = fields[1]
		}
	}

	fmt.Printf("%s %d\n", oldestName, maxAge)
}
