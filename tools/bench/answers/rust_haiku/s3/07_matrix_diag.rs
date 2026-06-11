fn main() {
    let matrix = [
        [5, 1, 2],
        [3, 6, 4],
        [7, 8, 9],
    ];

    let mut sum = 0;
    for i in 0..3 {
        sum += matrix[i][i];
    }

    println!("{}", sum);
}
