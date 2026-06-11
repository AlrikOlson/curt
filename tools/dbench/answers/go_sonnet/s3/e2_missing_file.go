package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func parseJSONFile(path string) (map[string]interface{}, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, err
	}
	return m, nil
}

func main() {
	name := "default"
	if m, err := parseJSONFile("missing.cfg"); err == nil {
		if v, ok := m["name"].(string); ok {
			name = v
		}
	}
	fmt.Println(name)

	port := 0
	if m, err := parseJSONFile("app.cfg"); err == nil {
		if v, ok := m["port"].(float64); ok {
			port = int(v)
		}
	}
	fmt.Println(port)
}
