package main

import "fmt"

func main() {
	s := "the quick brown fox jumps over the lazy dog"
	count := 0
	for _, ch := range s {
		if ch == 'a' || ch == 'e' || ch == 'i' || ch == 'o' || ch == 'u' {
			count++
		}
	}
	fmt.Println(count)
}
