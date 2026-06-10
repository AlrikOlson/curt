package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func load(p string) (map[string]any, error) {
	data, err := os.ReadFile(p)
	if err != nil {
		return nil, err
	}
	var cfg map[string]any
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, err
	}
	return cfg, nil
}

func main() {
	cfg, err := load("app.cfg")
	if err != nil {
		cfg = map[string]any{}
	}
	port, ok := cfg["port"]
	if !ok {
		port = 8080
	}
	fmt.Println(port)
}
