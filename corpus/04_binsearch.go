package main

import "fmt"

func bs(xs []int, t int) int {
	lo, hi := 0, len(xs)-1
	for lo <= hi {
		mid := (lo + hi) / 2
		if xs[mid] == t {
			return mid
		}
		if xs[mid] < t {
			lo = mid + 1
		} else {
			hi = mid - 1
		}
	}
	return -1
}

func main() {
	fmt.Println(bs([]int{1, 3, 5, 7, 9, 11}, 7))
}
