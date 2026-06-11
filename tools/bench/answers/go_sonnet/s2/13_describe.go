package main

import "fmt"

func describe(v interface{}) string {
	switch val := v.(type) {
	case int:
		return fmt.Sprintf("int %d", val+1)
	case float64:
		return fmt.Sprintf("float %g", val)
	case string:
		return fmt.Sprintf("str %d", len(val))
	}
	return ""
}

func main() {
	fmt.Println(describe(42))
	fmt.Println(describe(3.5))
	fmt.Println(describe("abc"))
}
