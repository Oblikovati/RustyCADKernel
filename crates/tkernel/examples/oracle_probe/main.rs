//! Text protocol driver for differential testing against the C++ OCCT executable.
use rustycad_tkernel::{
    Error,
    quantity::{Date, Period},
    standard::Guid,
};
use std::io::{self, BufRead};

mod message;
mod message_progress;
mod ncollection;
mod osd;
mod quantity;
mod resource;
mod storage;
mod units;

fn date(input: &mut std::str::SplitWhitespace<'_>) -> Result<Date, Error> {
    let values: Vec<i32> = input
        .take(8)
        .map(|s| s.parse().expect("integer input"))
        .collect();
    assert_eq!(values.len(), 8, "eight date fields required");
    Date::new(
        values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
    )
}
fn date_out(value: Date) -> String {
    let (mm, dd, yy, hh, mn, ss, mis, us) = value.values();
    format!("{mm} {dd} {yy} {hh} {mn} {ss} {mis} {us}")
}
fn period_out(value: Period) -> String {
    let (ss, us) = value.values();
    format!("{ss} {us}")
}
fn number(input: &mut std::str::SplitWhitespace<'_>) -> i32 {
    input
        .next()
        .expect("required field")
        .parse()
        .expect("integer input")
}
// Quantity operations fail with `Error`, GUID parsing with `Failure`; both print "Type: message".
fn evaluate(line: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = line.split_whitespace();
    let op = input.next().expect("operation");
    let output = match op {
        "period" => {
            let p = Period::from_components(
                number(&mut input),
                number(&mut input),
                number(&mut input),
                number(&mut input),
                number(&mut input),
                number(&mut input),
            )?;
            let (dd, hh, mn, ss, mis, us) = p.components();
            format!("{} {dd} {hh} {mn} {ss} {mis} {us}", period_out(p))
        }
        "period_math" => {
            let a = Period::new(number(&mut input), number(&mut input))?;
            let b = Period::new(number(&mut input), number(&mut input))?;
            format!(
                "{} {} {} {} {}",
                period_out(a.add(b)?),
                period_out(a.subtract(b)?),
                u8::from(a.is_equal(b)),
                u8::from(a.is_shorter(b)),
                u8::from(a.is_longer(b))
            )
        }
        "date" => {
            let a = date(&mut input)?;
            format!(
                "{} {}",
                date_out(a),
                period_out(a.difference(Date::default()))
            )
        }
        "date_add" | "date_sub" => {
            let a = date(&mut input)?;
            let p = Period::new(number(&mut input), number(&mut input))?;
            date_out(if op == "date_add" {
                a.add(p)?
            } else {
                a.subtract(p)?
            })
        }
        "date_diff" => {
            let a = date(&mut input)?;
            let b = date(&mut input)?;
            format!(
                "{} {} {} {}",
                period_out(a.difference(b)),
                u8::from(a.is_equal(b)),
                u8::from(a.is_earlier(b)),
                u8::from(a.is_later(b))
            )
        }
        "guid" => {
            let value = input.next().expect("GUID");
            let guid: Guid = if value == "EMPTY" { "" } else { value }.parse()?;
            let uuid = guid.to_uuid();
            let mut out = format!("{guid} {} {} {}", uuid.data1, uuid.data2, uuid.data3);
            for byte in uuid.data4 {
                out.push_str(&format!(" {byte}"));
            }
            out
        }
        other => ncollection::evaluate(other, &mut input)
            .or_else(|| message::evaluate(other, &mut input))
            .or_else(|| quantity::evaluate(other, &mut input))
            .or_else(|| resource::evaluate(other, &mut input))
            .or_else(|| message_progress::evaluate(other, &mut input))
            .or_else(|| osd::evaluate(other, &mut input))
            .or_else(|| units::evaluate(other, &mut input))
            .or_else(|| storage::evaluate(other, &mut input))
            .unwrap_or_else(|| panic!("unknown operation {other}")),
    };
    assert!(input.next().is_none(), "unexpected trailing fields");
    Ok(output)
}
fn main() {
    for line in io::stdin().lock().lines() {
        match evaluate(&line.expect("input line")) {
            Ok(value) => println!("OK {value}"),
            Err(error) => println!("ERR {error}"),
        }
    }
}
