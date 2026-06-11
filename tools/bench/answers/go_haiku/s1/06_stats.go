package main

import "fmt"

func main() {
	nums := []float64{4, 8, 15, 16, 23}
	min := nums[0]
	max := nums[0]
	sum := 0.0
	for _, num := range nums {
		if num < min {
			min = num
		}
		if num > max {
			max = num
		}
		sum += num
	}
	mean := sum / float64(len(nums))
	fmt.Println(int(min))
	fmt.Println(int(max))
	fmt.Println(mean)
}
