fn main() {
    let ages = [34i64, -2, 19, 150, 42, 27];
    let mut valid = 0usize;
    let mut invalid = 0usize;
    let mut sum = 0i64;
    for age in ages {
        if age >= 0 && age <= 120 {
            valid += 1;
            sum += age;
        } else {
            invalid += 1;
        }
    }
    println!("{}", valid);
    println!("{}", invalid);
    println!("{}", sum);
}
