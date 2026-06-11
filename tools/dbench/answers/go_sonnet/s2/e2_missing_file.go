package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func loadJSON(path string) map[string]interface{} {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil
	}
	var m map[string]interface{}
	if json.Unmarshal(b, &m) != nil {
		return nil
	}
	return m
}

func main() {
	cfg1 := loadJSON("missing.cfg")
	name := "default"
	if cfg1 != nil {
		if v, ok := cfg1["name"].(string); ok {
			name = v
		}
	}
	fmt.Println(name)

	cfg2 := loadJSON("app.cfg")
	port := 0
	if cfg2 != nil {
		if v, ok := cfg2["port"].(float64); ok {
			port = int(v)
		}
	}
	fmt.Println(port)
}
