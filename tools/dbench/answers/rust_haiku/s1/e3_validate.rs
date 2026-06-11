fn main() {
    let ages = [34, -2, 19, 150, 42, 27];
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut valid_sum = 0;

    for &age in &ages {
        if age >= 0 && age <= 120 {
            valid_count += 1;
            valid_sum += age;
        } else {
            invalid_count += 1;
        }
    }

    println!("{}", valid_count);
    println!("{}", invalid_count);
    println!("{}", valid_sum);
}
