use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;

fn main() {
    let l = TcpListener::bind("0.0.0.0:8080").unwrap();
    for c in l.incoming() {
        thread::spawn(move || {
            let mut c = c.unwrap();
            let r = BufReader::new(c.try_clone().unwrap());
            for ln in r.lines() {
                let _ = writeln!(c, "{}", ln.unwrap().to_uppercase());
            }
        });
    }
}
