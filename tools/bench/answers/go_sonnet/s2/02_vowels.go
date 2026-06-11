package main

import "fmt"

func main() {
	s := "the quick brown fox jumps over the lazy dog"
	count := 0
	for _, ch := range s {
		switch ch {
		case 'a', 'e', 'i', 'o', 'u':
			count++
		}
	}
	fmt.Println(count)
}
