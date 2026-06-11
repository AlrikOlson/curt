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
	var result []string

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
		if age > 25 {
			result = append(result, fields[1])
		}
	}

	fmt.Println(strings.Join(result, " "))
}
