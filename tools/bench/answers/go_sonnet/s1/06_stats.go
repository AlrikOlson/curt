package main

import "fmt"

func main() {
	nums := []float64{4, 8, 15, 16, 23}
	min, max := nums[0], nums[0]
	sum := 0.0
	for _, n := range nums {
		if n < min {
			min = n
		}
		if n > max {
			max = n
		}
		sum += n
	}
	mean := sum / float64(len(nums))
	fmt.Println(min)
	fmt.Println(max)
	fmt.Println(mean)
}
