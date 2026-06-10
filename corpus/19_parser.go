package main

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

type tok struct {
	num float64
	sym string
}

func lex(s string) []tok {
	s = strings.ReplaceAll(s, "(", " ( ")
	s = strings.ReplaceAll(s, ")", " ) ")
	ts := []tok{}
	for _, w := range strings.Fields(s) {
		if n, err := strconv.ParseFloat(w, 64); err == nil {
			ts = append(ts, tok{num: n})
		} else {
			ts = append(ts, tok{sym: w})
		}
	}
	return ts
}

func expr(ts []tok) (float64, []tok) {
	v, r := term(ts)
	for len(r) > 0 && (r[0].sym == "+" || r[0].sym == "-") {
		v2, r2 := term(r[1:])
		if r[0].sym == "+" {
			v += v2
		} else {
			v -= v2
		}
		r = r2
	}
	return v, r
}

func term(ts []tok) (float64, []tok) {
	v, r := factor(ts)
	for len(r) > 0 && (r[0].sym == "*" || r[0].sym == "/") {
		v2, r2 := factor(r[1:])
		if r[0].sym == "*" {
			v *= v2
		} else {
			v /= v2
		}
		r = r2
	}
	return v, r
}

func factor(ts []tok) (float64, []tok) {
	if ts[0].sym == "(" {
		v, r := expr(ts[1:])
		return v, r[1:]
	}
	return ts[0].num, ts[1:]
}

func main() {
	v, _ := expr(lex(os.Args[1]))
	fmt.Println(v)
}
