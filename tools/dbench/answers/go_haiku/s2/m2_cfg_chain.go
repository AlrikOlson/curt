package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	data, _ := os.ReadFile("app.cfg")
	var m map[string]interface{}
	json.Unmarshal(data, &m)

	mode := "prod"
	if d, ok := m["debug"].(bool); ok && d {
		mode = "debug"
	}

	host := "localhost"
	if h, ok := m["host"].(string); ok && h != "" {
		host = h
	}

	port := 0
	if p, ok := m["port"].(float64); ok {
		port = int(p)
	}

	fmt.Printf("%s %s:%d\n", mode, host, port)
}
