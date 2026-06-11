package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func main() {
	var config map[string]interface{}
	data, err := os.ReadFile("missing.cfg")
	if err != nil {
		fmt.Println("default")
	} else {
		json.Unmarshal(data, &config)
		if v, ok := config["name"]; ok {
			fmt.Println(v)
		} else {
			fmt.Println("default")
		}
	}

	port := 0
	data, err = os.ReadFile("app.cfg")
	if err == nil {
		var cfg map[string]interface{}
		if json.Unmarshal(data, &cfg) == nil {
			if p, ok := cfg["port"]; ok {
				if pval, ok := p.(float64); ok {
					port = int(pval)
				}
			}
		}
	}
	fmt.Println(port)
}
