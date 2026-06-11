package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	missing := readJSON("missing.cfg")
	if n, ok := missing["name"].(string); ok {
		fmt.Println(n)
	} else {
		fmt.Println("default")
	}

	app := readJSON("app.cfg")
	port := 0
	if p, ok := app["port"].(float64); ok {
		port = int(p)
	}
	fmt.Println(port)
}

func readJSON(path string) map[string]interface{} {
	data, err := os.ReadFile(path)
	if err != nil {
		return map[string]interface{}{}
	}
	var m map[string]interface{}
	json.Unmarshal(data, &m)
	return m
}
