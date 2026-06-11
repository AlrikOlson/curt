package main

import "fmt"

func main() {
	s := "aaabbcccca"
	result := ""
	i := 0
	for i < len(s) {
		ch := s[i]
		count := 1
		for i+count < len(s) && s[i+count] == ch {
			count++
		}
		result += fmt.Sprintf("%c%d", ch, count)
		i += count
	}
	fmt.Println(result)
}
