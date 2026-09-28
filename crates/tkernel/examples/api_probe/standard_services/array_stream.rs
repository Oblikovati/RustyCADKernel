//! Rust side of the Standard_ArrayStreamBuffer case (tkernel_standard_services_array_stream.cxx).
use crate::common::{decode, hex, next_bytes};
use rustycad_tkernel::standard::ArrayStreamBuffer;
use std::io::SeekFrom;
use std::str::SplitWhitespace;

fn byte_text(byte: Option<u8>) -> String {
    byte.map_or_else(|| "eof".into(), |byte| byte.to_string())
}

/// "services_array_stream <hex bytes> <token>...": see the native case for the tokens.
pub fn array_stream_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = next_bytes(input);
    let tokens: Vec<&str> = input.collect();
    // Every Init payload must outlive the borrowing buffer, so decode them up front.
    let payloads: Vec<Vec<u8>> = tokens
        .iter()
        .filter_map(|token| token.strip_prefix('i').map(decode))
        .collect();
    let mut payload = payloads.iter();
    let mut buffer = ArrayStreamBuffer::new(&first);
    let answers: Vec<String> = tokens
        .iter()
        .map(|token| {
            if token.starts_with('i') {
                buffer.init(payload.next().expect("decoded payload"));
                return "-".into();
            }
            step(&mut buffer, token)
        })
        .collect();
    answers.join(" ")
}

fn step(buffer: &mut ArrayStreamBuffer<'_>, token: &str) -> String {
    let (operation, arguments) = token.split_at(1);
    match operation {
        "g" => {
            let mut read = vec![0; arguments.parse().expect("count")];
            let count = buffer.xsgetn(&mut read);
            hex(&read[..count])
        }
        "c" => byte_text(buffer.peek()),
        "b" => byte_text(buffer.uflow()),
        "u" => byte_text(buffer.put_back(None)),
        "p" => byte_text(buffer.put_back(Some(arguments.parse::<i32>().expect("byte") as u8))),
        "a" => buffer.available().to_string(),
        _ => seek(buffer, operation, arguments.parse().expect("offset")),
    }
}

fn seek(buffer: &mut ArrayStreamBuffer<'_>, operation: &str, offset: i64) -> String {
    let target = match operation {
        "B" | "P" => SeekFrom::Start(u64::try_from(offset).expect("non-negative start offset")),
        "C" => SeekFrom::Current(offset),
        _ => SeekFrom::End(offset),
    };
    buffer
        .seek_occt(target)
        .map_or_else(|_| "err".into(), |position| position.to_string())
}
