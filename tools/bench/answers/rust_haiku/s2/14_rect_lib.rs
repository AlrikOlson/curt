struct Rectangle {
    width: f64,
    height: f64,
}

fn area(rect: &Rectangle) -> f64 {
    rect.width * rect.height
}

fn perimeter(rect: &Rectangle) -> f64 {
    2.0 * (rect.width + rect.height)
}

fn scale(rect: &Rectangle, factor: f64) -> Rectangle {
    Rectangle {
        width: rect.width * factor,
        height: rect.height * factor,
    }
}

fn main() {
    let rect = Rectangle {
        width: 3.0,
        height: 4.0,
    };

    println!("{}", area(&rect));
    println!("{}", perimeter(&rect));

    let scaled = scale(&rect, 2.0);
    println!("{}", area(&scaled));
}
