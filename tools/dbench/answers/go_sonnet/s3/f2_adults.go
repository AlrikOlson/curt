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

	var result []string
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		tok := strings.Fields(sc.Text())
		if len(tok) != 3 {
			continue
		}
		age, err := strconv.Atoi(tok[2])
		if err != nil {
			continue
		}
		if age > 25 {
			result = append(result, tok[1])
		}
	}
	fmt.Println(strings.Join(result, " "))
}
