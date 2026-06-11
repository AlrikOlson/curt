fn main() {
    let ages = [34, -2, 19, 150, 42, 27];
    let mut valid = 0;
    let mut invalid = 0;
    let mut vsum = 0;

    for &age in &ages {
        if age >= 0 && age <= 120 {
            valid += 1;
            vsum += age;
        } else {
            invalid += 1;
        }
    }

    println!("{}", valid);
    println!("{}", invalid);
    println!("{}", vsum);
}
