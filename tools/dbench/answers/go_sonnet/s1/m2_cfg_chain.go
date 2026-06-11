package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	data, err := os.ReadFile("app.cfg")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var cfg map[string]interface{}
	if err := json.Unmarshal(data, &cfg); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	mode := "prod"
	if debug, ok := cfg["debug"].(bool); ok && debug {
		mode = "debug"
	}

	host := "localhost"
	if h, ok := cfg["host"].(string); ok && h != "" {
		host = h
	}

	port := 0
	if p, ok := cfg["port"].(float64); ok {
		port = int(p)
	}

	fmt.Printf("%s %s:%d\n", mode, host, port)
}
