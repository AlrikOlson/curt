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

	cnt, ageTotal := 0, 0
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
		cnt++
		ageTotal += age
	}
	fmt.Println(cnt)
	avg := 0
	if cnt > 0 {
		avg = ageTotal / cnt
	}
	fmt.Println(avg)
}
