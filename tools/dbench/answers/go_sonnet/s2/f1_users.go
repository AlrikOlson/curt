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

	var count, ageSum int
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
		count++
		ageSum += age
	}
	fmt.Println(count)
	avg := 0
	if count > 0 {
		avg = ageSum / count
	}
	fmt.Println(avg)
}
