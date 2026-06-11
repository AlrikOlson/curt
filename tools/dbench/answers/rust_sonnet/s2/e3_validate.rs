fn main() {
    let ages = [34i64, -2, 19, 150, 42, 27];
    let (valid, invalid, sum) = ages.iter().fold((0usize, 0usize, 0i64), |(v, i, s), &a| {
        if a >= 0 && a <= 120 {
            (v + 1, i, s + a)
        } else {
            (v, i + 1, s)
        }
    });
    println!("{}", valid);
    println!("{}", invalid);
    println!("{}", sum);
}
