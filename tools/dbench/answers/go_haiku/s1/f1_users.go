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
	count := 0
	sumAge := 0

	for scanner.Scan() {
		fields := strings.Fields(scanner.Text())
		if len(fields) != 3 {
			continue
		}
		age, err := strconv.Atoi(fields[2])
		if err != nil {
			continue
		}
		count++
		sumAge += age
	}

	fmt.Println(count)
	if count > 0 {
		fmt.Println(sumAge / count)
	}
}
