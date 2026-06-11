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
	oldestName := ""
	oldestAge := -1

	for scanner.Scan() {
		line := scanner.Text()
		fields := strings.Fields(line)
		if len(fields) != 3 {
			continue
		}
		age, err := strconv.Atoi(fields[2])
		if err != nil {
			continue
		}
		if age > oldestAge {
			oldestAge = age
			oldestName = fields[1]
		}
	}

	fmt.Printf("%s %d\n", oldestName, oldestAge)
}
