package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	buf, _ := os.ReadFile("app.cfg")
	var config map[string]interface{}
	json.Unmarshal(buf, &config)

	mode := "prod"
	if debug, ok := config["debug"].(bool); ok && debug {
		mode = "debug"
	}

	host := "localhost"
	if h, ok := config["host"].(string); ok && h != "" {
		host = h
	}

	port := 0
	if p, ok := config["port"].(float64); ok {
		port = int(p)
	}

	fmt.Printf("%s %s:%d\n", mode, host, port)
}
