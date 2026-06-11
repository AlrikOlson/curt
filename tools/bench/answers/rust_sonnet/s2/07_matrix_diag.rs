fn main() {
    let m = [[5, 1, 2], [3, 6, 4], [7, 8, 9]];
    let sum: i64 = (0..3).map(|i| m[i][i]).sum();
    println!("{}", sum);
}
