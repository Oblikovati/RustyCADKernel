//! Exact parser positions, failures, partial writes, and translated vertex coordinates.
use rustycad_tkbrep::{brep::Tool, topods::Vertex};
use rustycad_tkernel::standard::{
    Failure,
    dump::{self, DumpStream},
};
use rustycad_tkmath::{
    gp::{Point3, Transform3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::MakeVertex;
use std::io::{self, BufRead};
fn unhex(text: &str) -> String {
    if text == "-" {
        return String::new();
    }
    String::from_utf8(
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect(),
    )
    .unwrap()
}
fn hex(text: &[u8]) -> String {
    if text.is_empty() {
        return "-".to_owned();
    }
    text.iter().map(|b| format!("{b:02x}")).collect()
}
fn parse(op: &str, input: &mut std::str::SplitWhitespace<'_>) {
    let mut pos: i32 = input.next().unwrap().parse().unwrap();
    let text = unhex(input.next().unwrap());
    let mut coordinates = [91.0, 92.0, 93.0];
    let mut value = String::new();
    let result: Result<bool, Failure> = match op {
        "vector" => dump::init_real_values(&text, &mut pos, &mut coordinates),
        "key" => dump::process_stream_name(&text, "gp_Pnt", &mut pos),
        "point" => {
            let mut point = Point3::new(91.0, 92.0, 93.0);
            let result = point.init_from_json(&text, &mut pos);
            coordinates = *point.coordinates();
            result
        }
        _ => dump::init_value(&text, &mut pos).map(|v| {
            value = hex(v.as_bytes());
            true
        }),
    };
    match result {
        Ok(ok) => print!(
            "ok {} {}",
            i32::from(ok),
            if value.is_empty() { "-" } else { &value }
        ),
        Err(e) => print!("err {} {}", e.exception_type(), hex(e.message().as_bytes())),
    }
    println!(
        " {} {} {} {}",
        pos,
        coordinates[0].to_bits(),
        coordinates[1].to_bits(),
        coordinates[2].to_bits()
    );
}
fn geometry(op: &str, input: &mut std::str::SplitWhitespace<'_>) {
    let coordinates: [f64; 3] = std::array::from_fn(|_| input.next().unwrap().parse().unwrap());
    if op == "dump" {
        let mut stream = DumpStream::default();
        Point3::new(coordinates[0], coordinates[1], coordinates[2])
            .dump_json(&mut stream)
            .unwrap();
        println!("{}", hex(stream.bytes()));
        return;
    }
    let power: i32 = input.next().unwrap().parse().unwrap();
    let transform = Transform3::translation(coordinates).powered(power).unwrap();
    let builder = MakeVertex::new(Point3::new(0.125, -0.25, 0.5));
    let moved = builder
        .shape()
        .moved(&Location::new(transform), false)
        .unwrap();
    let point = Tool::point(&Vertex::unchecked(moved)).unwrap();
    println!(
        "{} {} {}",
        point.x().to_bits(),
        point.y().to_bits(),
        point.z().to_bits()
    );
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut input = line.split_whitespace();
        let op = input.next().unwrap();
        if matches!(op, "dump" | "translation") {
            geometry(op, &mut input);
        } else {
            parse(op, &mut input);
        }
    }
}
