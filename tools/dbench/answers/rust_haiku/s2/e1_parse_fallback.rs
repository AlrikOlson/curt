fn main() {
    let items = ["12", "x", "7", "-", "30"];
    let mut total = 0;
    let mut unparseable = 0;

    for item in &items {
        match item.parse::<i32>() {
            Ok(num) => total += num,
            Err(_) => unparseable += 1,
        }
    }

    println!("{}", total);
    println!("{}", unparseable);
}
