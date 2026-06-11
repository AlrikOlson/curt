package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	f, _ := os.Open("users.txt")
	defer f.Close()

	sc := bufio.NewScanner(f)
	cnt := 0
	sum := 0

	for sc.Scan() {
		parts := strings.Fields(sc.Text())
		if len(parts) != 3 {
			continue
		}
		age, err := strconv.Atoi(parts[2])
		if err != nil {
			continue
		}
		cnt++
		sum += age
	}

	fmt.Println(cnt)
	if cnt > 0 {
		fmt.Println(sum / cnt)
	}
}
