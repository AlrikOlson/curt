package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	f, err := os.Open("users.txt")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	defer f.Close()

	count := 0
	sumAge := 0
	scanner := bufio.NewScanner(f)
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
	} else {
		fmt.Println(0)
	}
}
