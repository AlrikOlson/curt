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
	validCount := 0
	ageSum := 0

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
		validCount++
		ageSum += age
	}

	fmt.Println(validCount)
	if validCount > 0 {
		fmt.Println(ageSum / validCount)
	}
}
