package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	data, _ := os.ReadFile("app.cfg")
	var cfg map[string]interface{}
	json.Unmarshal(data, &cfg)

	mode := "prod"
	if debug, ok := cfg["debug"].(bool); ok && debug {
		mode = "debug"
	}

	host := "localhost"
	if h, ok := cfg["host"].(string); ok {
		host = h
	}

	port := 0
	if p, ok := cfg["port"].(float64); ok {
		port = int(p)
	}

	fmt.Printf("%s %s:%d\n", mode, host, port)
}
