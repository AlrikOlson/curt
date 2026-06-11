struct Rect {
    width: f64,
    height: f64,
}

fn area(r: &Rect) -> f64 {
    r.width * r.height
}

fn perimeter(r: &Rect) -> f64 {
    2.0 * (r.width + r.height)
}

fn scale(r: &Rect, factor: f64) -> Rect {
    Rect { width: r.width * factor, height: r.height * factor }
}

fn main() {
    let r = Rect { width: 3.0, height: 4.0 };
    println!("{}", area(&r));
    println!("{}", perimeter(&r));
    let r2 = scale(&r, 2.0);
    println!("{}", area(&r2));
}
