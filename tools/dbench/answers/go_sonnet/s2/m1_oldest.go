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

	maxAge := -1
	maxName := ""
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		parts := strings.Fields(sc.Text())
		if len(parts) != 3 {
			continue
		}
		age, err := strconv.Atoi(parts[2])
		if err != nil {
			continue
		}
		if age > maxAge {
			maxAge = age
			maxName = parts[1]
		}
	}
	fmt.Printf("%s %d\n", maxName, maxAge)
}
