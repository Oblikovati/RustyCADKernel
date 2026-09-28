//! Differential numeric-prefix probe; records exact IEEE bits and byte offsets.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin line");
        let (value, end) = rustycad_tkernel::standard::strtod(&line);
        println!("{:016x} {end}", value.to_bits());
    }
}
