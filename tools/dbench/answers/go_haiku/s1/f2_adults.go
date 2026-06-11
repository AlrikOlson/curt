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
	var names []string

	for scanner.Scan() {
		fields := strings.Fields(scanner.Text())
		if len(fields) != 3 {
			continue
		}
		age, err := strconv.Atoi(fields[2])
		if err != nil {
			continue
		}
		if age > 25 {
			names = append(names, fields[1])
		}
	}

	fmt.Println(strings.Join(names, " "))
}
