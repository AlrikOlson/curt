package main

import (
	"bufio"
	"net"
	"strings"
)

func handle(c net.Conn) {
	defer c.Close()
	s := bufio.NewScanner(c)
	for s.Scan() {
		c.Write([]byte(strings.ToUpper(s.Text()) + "\n"))
	}
}

func main() {
	l, _ := net.Listen("tcp", ":8080")
	for {
		c, _ := l.Accept()
		go handle(c)
	}
}
