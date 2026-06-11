package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	var cfg1 map[string]interface{}
	buf1, err1 := os.ReadFile("missing.cfg")
	if err1 != nil {
		fmt.Println("default")
	} else {
		json.Unmarshal(buf1, &cfg1)
		if val, exists := cfg1["name"]; exists {
			fmt.Println(val)
		} else {
			fmt.Println("default")
		}
	}

	var cfg2 map[string]interface{}
	portVal := 0
	buf2, err2 := os.ReadFile("app.cfg")
	if err2 == nil {
		json.Unmarshal(buf2, &cfg2)
		if p, exists := cfg2["port"]; exists {
			if pf, ok := p.(float64); ok {
				portVal = int(pf)
			}
		}
	}
	fmt.Println(portVal)
}
