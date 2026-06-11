fn main() {
    let ages = [34, -2, 19, 150, 42, 27];
    let mut ok = 0;
    let mut bad = 0;
    let mut ok_sum = 0;

    for age in ages {
        if age >= 0 && age <= 120 {
            ok += 1;
            ok_sum += age;
        } else {
            bad += 1;
        }
    }

    println!("{}", ok);
    println!("{}", bad);
    println!("{}", ok_sum);
}
