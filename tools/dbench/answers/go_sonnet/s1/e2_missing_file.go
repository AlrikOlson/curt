package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func readConfig(filename string) map[string]interface{} {
	data, err := os.ReadFile(filename)
	if err != nil {
		return nil
	}
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err != nil {
		return nil
	}
	return m
}

func main() {
	m1 := readConfig("missing.cfg")
	name := "default"
	if m1 != nil {
		if v, ok := m1["name"].(string); ok {
			name = v
		}
	}
	fmt.Println(name)

	m2 := readConfig("app.cfg")
	port := 0
	if m2 != nil {
		if v, ok := m2["port"].(float64); ok {
			port = int(v)
		}
	}
	fmt.Println(port)
}
