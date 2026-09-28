//! Stream-level storage operations (twin of `tkernel_storage_streams_probe.cxx`).
use super::common::{ascii_hex, double, float, hex, unhex};
use rustycad_tkernel::fsd::Base64;
use rustycad_tkernel::storage::{DriverBase, IoStream, Schema, format_general, version};
use rustycad_tkernel::tcollection::AsciiString;
use std::str::SplitWhitespace;

/// The output of a stream operation, or `None` when `op` belongs elsewhere.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "stor_fmt" | "stor_fmtf" => format_probe(op, input),
        "stor_stream" => stream_script(input),
        "stor_magic" => {
            let mut stream = IoStream::from_bytes(unhex(input.next().expect("bytes")));
            ascii_hex(&DriverBase::read_magic_number(&mut stream))
        }
        "stor_b64e" | "stor_b64d" | "stor_b64ec" | "stor_b64dc" => base64(op, input),
        "stor_ver" => String::from_utf8_lossy(version().as_bytes()).into_owned(),
        "stor_migr" => {
            let old = AsciiString::from_bytes(unhex(input.next().expect("name")));
            let mut renamed = AsciiString::new("unchanged");
            let found = Schema::check_type_migration(&old, &mut renamed);
            format!("{} {}", u8::from(found), ascii_hex(&renamed))
        }
        _ => return None,
    })
}

fn format_probe(op: &str, input: &mut SplitWhitespace<'_>) -> String {
    let precision: usize = input.next().expect("precision").parse().expect("integer");
    let bits = input.next().expect("bits");
    let value = if op == "stor_fmt" {
        double(bits)
    } else {
        f64::from(float(bits))
    };
    format_general(value, precision)
}

/// The extraction targets of one script.
#[derive(Default)]
struct Values {
    byte: u8,
    int: i32,
    short: i16,
    unsigned_short: u16,
    flag: bool,
}

fn stream_script(input: &mut SplitWhitespace<'_>) -> String {
    let mut stream = IoStream::from_bytes(unhex(input.next().expect("bytes")));
    let mut values = Values::default();
    let mut out = String::new();
    for op in input.by_ref() {
        out += &stream_op(op, &mut stream, &mut values);
        out += &format!(":{}", stream.rdstate());
    }
    out
}

fn stream_op(op: &str, stream: &mut IoStream, values: &mut Values) -> String {
    match op {
        "g" => {
            stream.get(&mut values.byte);
            format!(" g{}:{}", values.byte, stream.gcount())
        }
        "i" => {
            stream.extract_int(&mut values.int);
            format!(" i{}", values.int)
        }
        "s" => {
            stream.extract_short(&mut values.short);
            format!(" s{}", values.short)
        }
        "u" => {
            stream.extract_unsigned_short(&mut values.unsigned_short);
            format!(" u{}", values.unsigned_short)
        }
        "b" => {
            stream.extract_bool(&mut values.flag);
            format!(" b{}", u8::from(values.flag))
        }
        "w" => {
            let word = stream.extract_word(100);
            let length = word.iter().position(|byte| *byte == 0).unwrap_or(word.len());
            format!(" w{}", hex(&word[..length]))
        }
        "tg" => format!(" tg{}", stream.tellg()),
        "tp" => format!(" tp{}", stream.tellp()),
        "c" => {
            stream.clear(0);
            " c".into()
        }
        _ => sized_stream_op(op, stream),
    }
}

fn sized_stream_op(op: &str, stream: &mut IoStream) -> String {
    let (kind, size) = op.split_at(1);
    let size: i64 = size.parse().expect("size");
    match kind {
        "l" => {
            let mut line = Vec::new();
            stream.getline(&mut line, size, b'\n');
            let length = line.iter().position(|byte| *byte == 0).unwrap_or(line.len());
            format!(" l{}:{}", hex(&line[..length]), stream.gcount())
        }
        "x" => {
            stream.set_width(size);
            " x".into()
        }
        _ => {
            let mut bytes = vec![0; size as usize];
            stream.read(&mut bytes);
            let count = stream.gcount() as usize;
            format!(" r{}:{}", hex(&bytes[..count]), count)
        }
    }
}

fn base64(op: &str, input: &mut SplitWhitespace<'_>) -> String {
    let capacity: i64 = if op.ends_with('c') {
        input.next().expect("capacity").parse().expect("integer")
    } else {
        0
    };
    let source = unhex(input.next().expect("bytes"));
    match op {
        "stor_b64e" => ascii_hex(&Base64::encode(&source)),
        "stor_b64d" => {
            let buffer = Base64::decode(&source);
            match buffer.get() {
                None => "null".into(),
                Some(buffer) => format!(
                    "{} {} {}",
                    u8::from(buffer.is_empty()),
                    buffer.size(),
                    hex(buffer.as_slice())
                ),
            }
        }
        _ => {
            let mut target = vec![b'*'; capacity.max(0) as usize];
            let destination = (capacity >= 0).then_some(target.as_mut_slice());
            let result = if op == "stor_b64ec" {
                Base64::encode_into(destination, &source)
            } else {
                Base64::decode_into(destination, &source)
            };
            format!("{result} {}", hex(&target))
        }
    }
}
